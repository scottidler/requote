#![allow(unused_imports)]

use chumsky::prelude::*;
use chumsky::error::Simple;
use chumsky::text::whitespace;
use clap::{Parser as ClapParser, ValueEnum};
use eyre::Result;
use std::fs;
use std::{path::Path, str::FromStr};
use log::{debug, info, warn, error};
use env_logger;

#[derive(Debug, Clone, ValueEnum)]
enum Mode {
    Single,
    Double,
}

#[derive(ClapParser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(value_parser)]
    path: String,

    #[clap(short, long)]
    recursive: bool,

    #[clap(short = 'm', long, default_value_t = Mode::Single, value_enum)]
    mode: Mode,

    #[clap(short, long, default_value_t = false)]
    overwrite: bool,
}

fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();
    println!("Starting with arguments: {:?}", args);

    let path = Path::new(&args.path);
    process_path(path, args.recursive, &args.mode, args.overwrite)?;
    Ok(())
}

fn process_path(path: &Path, recursive: bool, mode: &Mode, overwrite: bool) -> Result<()> {
    debug!("Processing path: {:?}", path);
    if path.is_dir() && recursive {
        debug!("Directory found, processing recursively");
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            process_path(&entry.path(), recursive, mode, overwrite)?;
        }
    } else if path.is_file() {
        debug!("File found, processing: {:?}", path);
        process_file(path, mode, overwrite)?;
    }
    Ok(())
}

fn process_file(path: &Path, mode: &Mode, overwrite: bool) -> Result<()> {
    debug!("Processing file: {:?}", path);
    let content = fs::read_to_string(path)?;
    let processed_content = process_content(&content, mode);
    if overwrite {
        debug!("Overwriting original file");
        fs::write(path, processed_content)?;
    } else {
        let new_file_path = path.with_extension(format!("{}{}", path.extension().unwrap_or_default().to_str().unwrap_or(""), ".new"));
        debug!("Writing to new file: {:?}", new_file_path);
        fs::write(&new_file_path, processed_content)?;
    }
    Ok(())
}

fn process_content(content: &str, mode: &Mode) -> String {
    debug!("Processing content with mode: {:?}", mode);
    content.to_string()
}

