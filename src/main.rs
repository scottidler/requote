#![allow(unused_imports)]

use std::fmt::Display;
use std::fs;
use std::{path::Path, str::FromStr};
use log::{debug, info, warn, error};

use env_logger;
use chumsky::prelude::*;
use chumsky::error::Simple;
use chumsky::text::whitespace;
use clap::{Parser as ClapParser, ValueEnum};
use eyre::Result;

#[derive(Debug, Clone, PartialEq, ValueEnum)]
enum Mode {
    Single,
    Double,
}

#[derive(ClapParser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(value_parser, help = "path to file or directory")]
    path: String,

    #[clap(short, long, help = "process directories recursively")]
    recursive: bool,

    #[clap(short, long, default_value_t = Mode::Single, value_enum, help = "requote [default: double->single] OR single->double")]
    mode: Mode,

    #[clap(short, long, default_value_t = false, help = "override cases where requote would normally not make the change")]
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
    let processed_content = process_content(&content, mode)?;
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

fn regular_quote(mode: &Mode) -> Box<dyn Parser<char, String, Error = Simple<char>> + '_> {
    let escape = just('\\').then(any()).map(|(escape_char, c)| format!("{}{}", escape_char, c));
    let non_escape_char = filter(|c: &char| *c != '\\' && *c != '\"' && *c != '\'');

    let quote_parser = match mode {
        Mode::Single => just('\"').to('\''),
        Mode::Double => just('\'').to('\"'),
    };

    let content_parser = escape
        .or(non_escape_char.map(|c| c.to_string()))
        .repeated()
        .collect();

    // Use `move` to take ownership of `mode` in the closure
    let mode_clone = mode.clone(); // Clone `mode` since it needs to be used twice
    Box::new(
        quote_parser
            .ignore_then(content_parser)
            .then_ignore(quote_parser.clone()) // Clone `quote_parser` if necessary for reuse
            .map(move |content: String| match mode_clone {
                Mode::Single => format!("'{}'", content),
                Mode::Double => format!("\"{}\"", content),
            }),
    )
}

fn triple_quote(mode: &Mode) -> Box<dyn Parser<char, String, Error = Simple<char>>> {
    let triple_single_content = just("'''").ignore_then(none_of('\'').repeated()).then_ignore(just("'''"));
    let triple_double_content = just("\"\"\"").ignore_then(none_of('\"').repeated()).then_ignore(just("\"\"\""));

    match mode {
        Mode::Single => Box::new(
            triple_double_content
                .map(|chars| chars.into_iter().collect::<String>())
                .map(|content| format!("'''{}'''", content.replace("\\\"", "\"")))
        ),
        Mode::Double => Box::new(
            triple_single_content
                .map(|chars| chars.into_iter().collect::<String>())
                .map(|content| format!("\"\"\"{}\"\"\"", content.replace("\\'", "'")))
        ),
    }
}

fn process_content(content: &str, mode: &Mode) -> Result<String> {
    debug!("Processing content with mode: {:?}", mode);
    let regular_quote_parser = regular_quote(mode);
    let triple_quote_parser = triple_quote(mode);
    let parser = triple_quote_parser
        .or(regular_quote_parser)
        .or(any().map(|c: char| c.to_string()))
        .repeated()
        .then_ignore(end())
        .map(|parts: Vec<String>| parts.concat());
    parser.parse(content)
        .map_err(|e| eyre::eyre!("Parse error: {:?}", e))
}

