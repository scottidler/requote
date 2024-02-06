use chumsky::prelude::*;
use chumsky::Parser;
use chumsky::error::Simple;
use clap::Parser as ClapParser;
use eyre::Result;
use std::fs;
use std::path::Path;

#[derive(ClapParser, Debug)]
#[clap(author, version, about)]
struct Args {
    #[clap(value_parser)]
    path: String,

    #[clap(short, long)]
    recursive: bool,

    #[clap(short = 'R', long)]
    reverse: bool,

    #[clap(short, long, default_value_t = false)]
    overwrite: bool,
}

#[derive(Debug)]
enum Quote {
    Single(String),
    Double(String),
    Single3(String),
    Double3(String),
}

fn main() -> Result<()> {
    let args = Args::parse();
    let path = Path::new(&args.path);
    process_path(path, args.recursive, args.reverse, args.overwrite)?;
    Ok(())
}

fn process_path(path: &Path, recursive: bool, reverse: bool, overwrite: bool) -> Result<()> {
    if path.is_dir() && recursive {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                process_path(&path, recursive, reverse, overwrite)?;
            } else if path.is_file() {
                process_file(&path, reverse, overwrite)?;
            }
        }
    } else if path.is_file() {
        process_file(path, reverse, overwrite)?;
    }
    Ok(())
}

fn process_file(path: &Path, reverse: bool, overwrite: bool) -> Result<()> {
    let content = fs::read_to_string(path)?;
    let processed_content = process_content(&content, reverse);
    let new_file_path = path.with_extension(format!("{}{}", path.extension().unwrap_or_default().to_str().unwrap_or(""), ".new"));
    fs::write(&new_file_path, processed_content)?;
    if overwrite {
        fs::rename(&new_file_path, path)?;
    }
    Ok(())
}

fn process_content(content: &str, reverse: bool) -> String {
    // no op for now
    let quotes = find_quotes(content);
    for quote in quotes {
        println!("{:?}", quote);
    }
    content.to_string()
}

fn find_quotes(content: &str) -> Vec<Quote> {
    let single_quote = just::<_, _, Simple<char>>('\'')
        .ignore_then(take_until(just('\'')).map(|(chars, _): (Vec<char>, _)| chars.into_iter().collect::<String>()))
        .then_ignore(just('\''))
        .map(Quote::Single);

    let double_quote = just::<_, _, Simple<char>>('\"')
        .ignore_then(take_until(just('\"')).map(|(chars, _): (Vec<char>, _)| chars.into_iter().collect::<String>()))
        .then_ignore(just('\"'))
        .map(Quote::Double);

    let triple_single_quote = just::<_, _, Simple<char>>("'''")
        .ignore_then(take_until(just("'''")).map(|(chars, _): (Vec<char>, &'static str)| chars.into_iter().collect::<String>()))
        .then_ignore(just("'''"))
        .map(Quote::Single3);

    let triple_double_quote = just::<_, _, Simple<char>>("\"\"\"")
        .ignore_then(take_until(just("\"\"\"")).map(|(chars, _): (Vec<char>, &'static str)| chars.into_iter().collect::<String>()))
        .then_ignore(just("\"\"\""))
        .map(Quote::Double3);

    let parser = single_quote
        .or(double_quote)
        .or(triple_single_quote)
        .or(triple_double_quote)
        .repeated();

    parser.parse(content).unwrap_or_else(|errors| {
        errors.into_iter().for_each(|e| eprintln!("Error: {:?}", e));
        Vec::new()
    })
}
