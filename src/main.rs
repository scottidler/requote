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
    // Define parsers for content inside quotes excluding the quotes themselves
    let content_inside_quotes = filter(|&c: &char| true) // Accept all characters
        .repeated()
        .collect::<String>();

    // Define parsers for triple quotes
    let triple_quote_parser = |quote: char, opposite_quote: char| {
        just::<_, [char; 3], Simple<char>>([quote, quote, quote])
            .ignore_then(
                filter(move |&c| true) // Accept all characters for triple-quoted content
                    .repeated()
                    .collect::<String>()
            )
            .then_ignore(just::<_, [char; 3], Simple<char>>([quote, quote, quote]))
            .map(move |content: String| format!("{0}{1}{0}", if reverse { opposite_quote } else { quote }, content))
    };

    // Define a parser for single and double quotes considering the content
    let quote_parser = |quote: char, opposite_quote: char| {
        just::<_, char, Simple<char>>(quote)
            .ignore_then(content_inside_quotes.clone())
            .then_ignore(just::<_, char, Simple<char>>(quote))
            .map(move |content: String| {
                // Check if the content contains the opposite quote
                if content.contains(opposite_quote) {
                    format!("{0}{1}{0}", quote, content) // Do not change if contains opposite quote
                } else {
                    format!("{0}{1}{0}", if reverse { opposite_quote } else { quote }, content) // Convert otherwise
                }
            })
    };

    // Combine parsers
    let parser = choice((
        triple_quote_parser('\'', '\"'),
        triple_quote_parser('\"', '\''),
        quote_parser('\'', '\"'),
        quote_parser('\"', '\''),
        any().map(|c: char| c.to_string()),
    ))
    .repeated()
    .collect::<String>();

    parser.parse(content.chars().collect::<Vec<_>>())
          .unwrap_or_else(|e| {
              eprintln!("Error parsing content: {:?}", e);
              content.to_owned()
          })
}
