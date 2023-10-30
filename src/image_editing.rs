use std::cmp;

use image::{Pixel, ImageBuffer, GenericImage, imageops::{flip_horizontal, resize, FilterType::Lanczos3, flip_vertical}, ImageError, error::{ParameterError, ParameterErrorKind}, ImageResult, Rgba};
use imageproc::{geometric_transformations::{rotate_about_center, Interpolation, translate}, definitions::{Image, Clamp}, drawing::Canvas};
use conv::ValueInto;

use crate::{TranslationMatrix, TranslationRow};

pub enum Orientation {
  Horizontal,
  Vertical
}

pub fn append_images<'a, P, I>(iter: I, width_of_single_image: u32, height_of_single_image: u32, orientation: Orientation) -> Image<P>
where
  P: Pixel + Send + Sync + 'a,
  <P as Pixel>::Subpixel: Send + Sync,
  I: ExactSizeIterator<Item = &'a Image<P>>,
{
  let length = iter.len();
  let size_of_items: (u32, u32) = match orientation {
    Orientation::Horizontal => (width_of_single_image * length as u32, height_of_single_image),
    Orientation::Vertical => (width_of_single_image, height_of_single_image * length as u32),
};

  let horizontal_append = fun_name(width_of_single_image);

  let buf = iter
    .into_iter()
    .enumerate();
  match orientation {
    Orientation::Horizontal => buf.fold(ImageBuffer::new(size_of_items.0, size_of_items.1), horizontal_append),
    Orientation::Vertical => buf.fold(ImageBuffer::new(size_of_items.0, size_of_items.1), vertical_append(height_of_single_image)),
}
}

fn fun_name<P>(width_of_single_image: u32)
-> impl Fn(ImageBuffer<P, Vec<<P as Pixel>::Subpixel>>, (usize, &ImageBuffer<P, Vec<<P as Pixel>::Subpixel>>))
-> ImageBuffer<P, Vec<<P as Pixel>::Subpixel>>
where P: Pixel + Send + Sync {
  move |mut acc: ImageBuffer<P, Vec<<P as Pixel>::Subpixel>>, image: (usize, &ImageBuffer<P, Vec<<P as Pixel>::Subpixel>>)|{
    acc.copy_from(image.1, width_of_single_image * image.0 as u32, 0).unwrap();
    acc
  }
}

fn vertical_append<P>(height_of_single_image: u32)
-> impl Fn(ImageBuffer<P, Vec<<P as Pixel>::Subpixel>>, (usize, &ImageBuffer<P, Vec<<P as Pixel>::Subpixel>>)) 
-> ImageBuffer<P, Vec<<P as Pixel>::Subpixel>> 
where P: Pixel + Send + Sync {
  move |mut acc, image| {
    acc.copy_from(image.1, 0, height_of_single_image * image.0 as u32).unwrap();
    acc
  }
}

pub fn mask_image<P>(base_image: &Image<P>, masking_image: &Image<P>) -> ImageResult<Image<P>>
where
  P: Pixel + Send + Sync,
  <P as Pixel>::Subpixel: Send + Sync + Ord,
{
  let height = base_image.height();
  let width = base_image.width();
  if height != masking_image.height() && width != masking_image.width() {
    Err(ImageError::Parameter(ParameterError::from_kind(ParameterErrorKind::DimensionMismatch)))
  } else {
    let mask_pixel = |mut acc: Vec<<P as Pixel>::Subpixel> , pixel_pair: (&P, &P)| {
      let base_channels = pixel_pair.0.channels();
      let masking_alpha = pixel_pair.1.channels()[3];

      let res_channels = &[
        base_channels[0],
        base_channels[1],
        base_channels[2],
        base_channels[3] - masking_alpha.min(base_channels[3]),
      ];

      acc.extend_from_slice(res_channels);
      acc
    };
    let pixel_array: Vec<P::Subpixel> = base_image.pixels()
    .zip(masking_image.pixels())
    .fold(vec![], mask_pixel);

    let res = ImageBuffer::from_vec(width, height, pixel_array).unwrap();
    Ok(res)
  }
}

pub fn make_base_item_image<P>(item_image: &Image<P>, settings: &TranslationRow, default: &P) -> ImageResult<Image<P>>
where
  P: Pixel + Send + Sync + 'static,
  <P as Pixel>::Subpixel: Send + Sync + ValueInto<f32> + Clamp<f32>,
{
  let original_height = item_image.height();
  let original_width = item_image.width();

  // let max_x_translation: f32 = settings.row.iter().map(|translation| translation.0.abs()).max().unwrap() as f32;
  // let max_y_translation: f32 = settings.row.iter().map(|translation| translation.1.abs()).max().unwrap() as f32;
  // let scaling_factor_x = 1.0 - ((max_x_translation / original_width as f32) * 2.0);
  // let scaling_factor_y = 1.0 - ((max_y_translation / original_height as f32) * 2.0);
  // let scaled_width= (scaling_factor_x * item_image.width() as f32) as u32;
  // let scaled_height= (scaling_factor_y * item_image.height() as f32) as u32;
  // will need to figure out the farthest right/left/top/bottom after rotation non zero alpha pixel for auto scaling due to needing image specific data 

  // see if there is a way to conditionally move item_image
  let scaled_height= (settings.scaling_factor * item_image.height() as f32) as u32;
  let scaled_width= (settings.scaling_factor * item_image.width() as f32) as u32;
  
  let scaled_height_offset= (original_height - scaled_height) / 2;
  let scaled_width_offset= (original_width - scaled_width) / 2;
  
  
  let scaled_img = if settings.scaling_factor != 1.0 {
    resize(item_image, scaled_width, scaled_height, Lanczos3)
  } else {
    item_image.to_owned()
  };

  let mut res = ImageBuffer::new(original_width, original_height);
  res.copy_from(&scaled_img, scaled_width_offset,scaled_height_offset)?;

  res = if settings.mirror_x {
    flip_horizontal(&res)
  } else {
    res
  };

  res = if settings.mirror_y {
    flip_vertical(&res)
  } else {
    res
  };

  res = if settings.angle != 0.0 {
    rotate_about_center(&res, settings.angle, Interpolation::Bilinear, *default)
  } else {
    res
  };

  Ok(res)
}

pub fn generate_paperdoll<P>(character_image: &Image<P>, item_image: &Image<P>, translation_matrix: &TranslationMatrix, default: &P) -> ImageResult<Image<P>>
where
  P: Pixel + Send + Sync + 'static,
  <P as Pixel>::Subpixel: Send + Sync + ValueInto<f32> + Clamp<f32> + Ord,
{
  let buffer_width = 32;
  let buffer_height = 48;
  let expanded_starting_x = 0;
  let expanded_starting_y = (buffer_height as i32 - item_image.height() as i32).unsigned_abs() / 2 ;
  let matrix = &translation_matrix.matrix;
  

  
  let mut expanded_item_image = ImageBuffer::new(buffer_width, buffer_height);
  expanded_item_image.copy_from(item_image, expanded_starting_x, expanded_starting_y).unwrap();

  let rows = vec![
    translate_row_images(&matrix[0].row, make_base_item_image(&expanded_item_image, &matrix[0], default)?),
    translate_row_images(&matrix[1].row, make_base_item_image(&expanded_item_image, &matrix[1], default)?),
    translate_row_images(&matrix[2].row, make_base_item_image(&expanded_item_image, &matrix[2], default)?),
    translate_row_images(&matrix[3].row, make_base_item_image(&expanded_item_image, &matrix[3], default)?)];

  let res: Vec<ImageBuffer<P, Vec<<P as Pixel>::Subpixel>>> = rows.iter().map(|row| {
    let row_iter = row.iter();
    append_images(row_iter, expanded_item_image.width(), expanded_item_image.height(), Orientation::Horizontal)
  }).collect();

  let umasked_paperdoll = append_images(res.iter(), buffer_width * translation_matrix.matrix[0].row.len() as u32, buffer_height, Orientation::Vertical);

  let masked_paperdoll = mask_image(&umasked_paperdoll, character_image)?;
  Ok(masked_paperdoll)


  // for miror use projection with mirror matrix transform
}

fn get_max_of_all_translations(translation_matrix: &TranslationMatrix) -> i32
{
    translation_matrix
    .matrix
    .iter()
    .fold(0, |acc, val|
      cmp::max(
      val
      .row
      .iter()
      .fold(0, |max_in_row, row|
        cmp::max(max_in_row, row.0.abs())),acc))
}

fn translate_row_images<P>(translation_matrix: &[(i32, i32)], vertical_item: ImageBuffer<P, Vec<<P as Pixel>::Subpixel>>) -> Vec<ImageBuffer<P, Vec<<P as Pixel>::Subpixel>>>
where P: Pixel + Send + Sync + 'static
{
    translation_matrix
    .iter()
    .map(|val| translate(&vertical_item, *val))
    .collect()
}