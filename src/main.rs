mod cli;
mod image_editing;

use serde::Deserialize;
use std::{error::Error, io::{BufWriter, BufReader}, fs::File};
use crate::cli::parse_command_line;
use image::{io::Reader as ImageReader, Rgba, ImageOutputFormat};
use image_editing::generate_paperdoll;

#[derive(Deserialize)]
pub struct TranslationRow {
    angle: f32,
    scaling_factor: f32,
    mirror: bool,
    row: Vec<(i32, i32)>,
}

#[derive(Deserialize)]
pub struct TranslationMatrix {
    matrix: [TranslationRow; 4]
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = parse_command_line()?;

    let character_file_string = args.character_file.unwrap();
    let dynamic_character_image = image::open(character_file_string.to_str().unwrap())?;
    let character_image = dynamic_character_image.as_rgba8().unwrap();
    let output_destination = args.output_directory.unwrap();
    let settings_file_path = args.settings.unwrap();

    
    let settings: TranslationMatrix  = serde_json::from_reader(BufReader::new(File::open(settings_file_path)?))?;
    
    let default = &Rgba([0, 0, 0, 0]);
    // let translation_matrix = [
    //     vec![(-6,0), (-6,1), (-6,0), (-6,-2)],
    //     vec![(0,0), (-3,-1), (0,0), (2,0)],
    //     vec![(0,0), (0,1), (0,0), (0,-2)],
    //     vec![(0,0), (0,1), (0,0), (0,-2)]
    // ];

    if let Some(path) = args.item_file {
        let dynamic_item_image = ImageReader::open(path)?.decode()?;
        let item_image = dynamic_item_image.as_rgba8().unwrap();
        let paperdoll = generate_paperdoll(character_image, item_image, &settings, default);
        let writer: &mut BufWriter<File> = &mut BufWriter::new(File::create(output_destination.clone())?);
        let _ = paperdoll.write_to(writer, ImageOutputFormat::Png);
    }
    
    Ok(())
}


