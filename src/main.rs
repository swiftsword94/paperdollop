mod cli;
mod image_editing;

use serde::Deserialize;
use std::{error::Error, io::{BufWriter, BufReader}, fs::{File, read_dir}, path::Path};
use crate::cli::parse_command_line;
use image::{io::Reader as ImageReader, Rgba, ImageOutputFormat};
use image_editing::generate_paperdoll;

#[derive(Deserialize)]
pub struct TranslationRow {
    // How much to rotate counterclockwise. Any rotation incurs smoothing in the ouput image.
    angle: f32,
    // Value between 0 and 1 to normally used to help with offseting by an ammount that would cause the image to go out of bounds.
    scaling_factor: f32,
    // Boolean for mirroring against x axis.
    mirror_x: bool,
    // Boolean for mirroring against y axis.
    mirror_y: bool,
    // An array of Arrays indicating translation in the (X axis, Y axis).
    frames: Vec<(i32, i32)>,
}



#[derive(Deserialize)]
pub struct TranslationMatrix {
    matrix: [TranslationRow; 4]
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = parse_command_line()?;

    let character_file_string = args.character_file;
    let dynamic_character_image = image::open(character_file_string.to_str().unwrap())?;
    let character_image = dynamic_character_image.as_rgba8().unwrap();
    let output_destination = args.output_directory;
    let settings_file_path = args.settings;

    
    let settings: TranslationMatrix  = serde_json::from_reader(BufReader::new(File::open(settings_file_path)?))?;
    
    let default = &Rgba([0, 0, 0, 0]);

    if let Some(item_path) = args.item_file {
        generate_and_save_paperdoll(&item_path, character_image, &settings, default, &output_destination)
    } else {
        let dir = args.item_directory.unwrap();
        read_dir(dir)?
            .try_for_each(|dir_entry_result| {
                let entry = dir_entry_result?;
                let item_path = entry.path();
                generate_and_save_paperdoll(&item_path, character_image, &settings, default, &output_destination)
            })
    }
}

fn generate_and_save_paperdoll(item_path: &Path, character_image: &image::ImageBuffer<Rgba<u8>, Vec<u8>>, settings: &TranslationMatrix, default: &Rgba<u8>, output_destination: &Path) -> Result<(), Box<dyn Error>> {
    let dynamic_item_image = ImageReader::open(item_path)?.decode()?;
    let item_image = dynamic_item_image.as_rgba8().unwrap();
    let paperdoll = generate_paperdoll(character_image, item_image, settings, default)?;
    let item_name = item_path.file_name().unwrap();
    let ouput_file = output_destination.join(item_name);
    let writer: &mut BufWriter<File> = &mut BufWriter::new(File::create(ouput_file)?);
    match paperdoll.write_to(writer, ImageOutputFormat::Png) {
        Ok(_) => Ok(()),
        Err(err) => Err(Box::new(err)),
    }
}


