use std::f32::consts::{FRAC_PI_4, FRAC_PI_2};

use image::{Pixel, ImageBuffer, GenericImage, imageops::{flip_horizontal, resize}};
use imageproc::{geometric_transformations::{rotate_about_center, Interpolation, translate}, definitions::{Image, Clamp}};
use conv::ValueInto;

use crate::{TranslationMatrix, TranslationRow};

pub enum Orientation {
  Horizontal,
  Vertical
}

pub fn append_images<'a, P, I>(iter: I, width_of_single_image: u32, height_of_single_image: u32, orientation: Orientation) -> Image<P>
where
  P: Pixel + Send + Sync + 'a,
  <P as Pixel>::Subpixel: Send + Sync + ValueInto<f32> + Clamp<f32>,
  I: ExactSizeIterator<Item = &'a Image<P>>,
{
  let length = iter.len();
  let size_of_items: (u32, u32) = match orientation {
    Orientation::Horizontal => (width_of_single_image * length as u32, height_of_single_image),
    Orientation::Vertical => (width_of_single_image, height_of_single_image * length as u32),
};

  let horizontal_append = |mut acc: ImageBuffer<P, Vec<<P as Pixel>::Subpixel>>, image: (usize, &ImageBuffer<P, Vec<<P as Pixel>::Subpixel>>)|{
    acc.copy_from(image.1, width_of_single_image * image.0 as u32, 0).unwrap();
    acc
  };

  let vertical_append = |mut acc: ImageBuffer<P, Vec<<P as Pixel>::Subpixel>>, image: (usize, &ImageBuffer<P, Vec<<P as Pixel>::Subpixel>>)|{
    let x = acc.copy_from(image.1, 0, height_of_single_image * image.0 as u32);
    x.unwrap();
    acc
  };
//fold((0, 0),|acc, image| (acc.0 + image.width(), image.height()));
  let buf = iter
    .into_iter()
    .enumerate();
  match orientation {
    Orientation::Horizontal => buf.fold(ImageBuffer::new(size_of_items.0, size_of_items.1), horizontal_append),
    Orientation::Vertical => buf.fold(ImageBuffer::new(size_of_items.0, size_of_items.1), vertical_append),
}
}

pub fn make_base_item_image<P>(item_image: &Image<P>, settings: &TranslationRow, default: &P) -> Image<P>
where
  P: Pixel + Send + Sync + 'static,
  <P as Pixel>::Subpixel: Send + Sync + ValueInto<f32> + Clamp<f32>,
{
  // let mut item = item_image;
  let flip = &flip_horizontal(item_image);
  if settings.mirror {
    rotate_about_center(flip, settings.angle, Interpolation::Bilinear, *default)
  } else {
    rotate_about_center(item_image, settings.angle, Interpolation::Bilinear, *default)
  }
}


// in radians
const ANGLE_TO_VERTICAL: f32 = -FRAC_PI_4;

pub fn generate_paperdoll<P>(character_image: &Image<P>, item_image: &Image<P>, translation_matrix: &TranslationMatrix, default: &P) -> Image<P>
where
  P: Pixel + Send + Sync + 'static,
  <P as Pixel>::Subpixel: Send + Sync + ValueInto<f32> + Clamp<f32>,
{
  let buffer_width = 32;
  let buffer_height = 48;
  let expanded_starting_x = 0;
  let expanded_starting_y = (buffer_height - item_image.height()) / 2 ;
  let default_pixel = default.to_owned();
  let matrix = &translation_matrix.matrix;
  
  let mut expanded_item_image = ImageBuffer::new(buffer_width, buffer_height);
  expanded_item_image.copy_from(item_image, expanded_starting_x, expanded_starting_y).unwrap();

  let vertical_item = rotate_about_center(&expanded_item_image, ANGLE_TO_VERTICAL, Interpolation::Bilinear, default_pixel);

  let mirrored_item = flip_horizontal(&expanded_item_image);

  let rows = vec![
    translate_row_images(&matrix[0].row, make_base_item_image(&expanded_item_image, &matrix[0], default)),
    translate_row_images(&matrix[1].row, make_base_item_image(&expanded_item_image, &matrix[1], default)),
    translate_row_images(&matrix[2].row, make_base_item_image(&expanded_item_image, &matrix[2], default)),
    translate_row_images(&matrix[3].row, make_base_item_image(&ImageBuffer::new(buffer_width, buffer_height), &matrix[3], default))];

  let res: Vec<ImageBuffer<P, Vec<<P as Pixel>::Subpixel>>> = rows.iter().map(|row| {
    let row_iter = row.iter();
    append_images(row_iter, expanded_item_image.width(), expanded_item_image.height(), Orientation::Horizontal)
  }).collect();

  append_images(res.iter(), buffer_width * translation_matrix.matrix[0].row.len() as u32, buffer_height, Orientation::Vertical)
  // for miror use projection with mirror matrix transform
}

fn translate_row_images<P>(translation_matrix: &Vec<(i32, i32)>, vertical_item: ImageBuffer<P, Vec<<P as Pixel>::Subpixel>>) -> Vec<ImageBuffer<P, Vec<<P as Pixel>::Subpixel>>>
where P: Pixel + Send + Sync + 'static
{
    let first_row: Vec<ImageBuffer<P, Vec<<P as Pixel>::Subpixel>>> = translation_matrix
    .iter()
    .map(|val| translate(&vertical_item, *val))
    .collect();
    first_row
}