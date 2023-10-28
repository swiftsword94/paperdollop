use std::path::PathBuf;

use clap::{Parser, command};



#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Sets a custom config file
    #[arg(required = true, short, long, value_name = "FILE")]
    pub character_file: PathBuf,
    
    #[arg(short = 'i', long, value_name = "FILE")]
    pub item_file: Option<PathBuf>,
    
    #[arg(short = 'I', long, value_name = "DIRECTORY")]
    pub item_directory: Option<PathBuf>,
    
    #[arg(required = true, short, long, value_name = "DIRECTORY", )]
    pub output_directory: PathBuf,
    
    #[arg(required = true, short, long, value_name = "FILE", )]
    pub settings: PathBuf,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,
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