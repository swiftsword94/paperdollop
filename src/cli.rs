use std::path::PathBuf;

use clap::{Parser, command};



#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// The character file used for masking
    #[arg(required = true, short, long, value_name = "FILE")]
    pub character_file: PathBuf,
    
    /// a single item, weapon, shield etc...
    #[arg(short = 'i', long, value_name = "FILE")]
    pub item_file: Option<PathBuf>,
    
    /// a directory of items to create paperdolls from
    #[arg(short = 'I', long, value_name = "DIRECTORY")]
    pub item_directory: Option<PathBuf>,
    
    /// the directory to ouptut the paperdolls
    #[arg(required = true, short, long, value_name = "DIRECTORY")]
    pub output_directory: PathBuf,
    
    /// the path of the settings file to use for paperdoll configuration. 
    #[arg(required = true, short, long, value_name = "FILE")]
    pub settings: PathBuf,
}

pub fn parse_command_line() -> Result<Cli, String> {
  let cli = Cli::parse();
  
  let item_file = cli.item_file.as_ref();
  let item_directory = cli.item_directory.as_ref();
  

  if item_file.is_some() && item_directory.is_some() {
      Err(String::from("Pick only one. Item file or item directory."))
  } else if item_file.is_none() && item_directory.is_none() {
      Err(String::from("Item file/directory path is missing!"))
  } else {
      Ok(cli)
  }
}