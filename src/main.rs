use std::{fs, path::Path};
use clap::Parser;
use eyre::Result;
use regex::Regex;
use regex::Captures;


#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(value_parser)]
    path: String,

    #[clap(short, long, action)]
    recursive: bool,

    #[clap(short = 'R', long)]
    reverse: bool,
}

fn process_path(path: &Path, recursive: bool, reverse: bool) -> Result<()> {
    if path.is_dir() && recursive {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                process_path(&path, recursive, reverse)?;
            } else {
                process_file(&path, reverse)?;
            }
        }
    } else if path.is_file() {
        process_file(path, reverse)?;
    }
    Ok(())
}

fn process_file(path: &Path, reverse: bool) -> Result<()> {
    let content = fs::read_to_string(path)?;
    let processed_content = process_content(&content, reverse);
    fs::write(path, processed_content)?;
    Ok(())
}

fn process_content(content: &str, reverse: bool) -> String {
    let triple_single_regex = Regex::new(r"'''[\s\S]*?'''").unwrap();
    let triple_double_regex = Regex::new(r#""""[\s\S]*?""""#).unwrap();
    let single_quote_regex = Regex::new(r"'([^']*)'").unwrap();
    let double_quote_regex = Regex::new(r#""([^"]*)""#).unwrap();

    let mut processed_content = content.to_string();

    if reverse {
        processed_content = triple_single_regex.replace_all(&processed_content, |caps: &Captures| {
            format!("\"\"\"{}\"\"\"", &caps[0][3..caps[0].len()-3])
        }).to_string();

        processed_content = single_quote_regex.replace_all(&processed_content, |caps: &Captures| {
            format!("\"{}\"", &caps[1])
        }).to_string();
    } else {
        processed_content = triple_double_regex.replace_all(&processed_content, |caps: &Captures| {
            format!("'''{}'''", &caps[0][3..caps[0].len()-3])
        }).to_string();

        processed_content = double_quote_regex.replace_all(&processed_content, |caps: &Captures| {
            format!("'{}'", &caps[1])
        }).to_string();
    }

    processed_content
}

fn main() -> Result<()> {
    let args = Args::parse();
    let path = Path::new(&args.path);
    process_path(path, args.recursive, args.reverse)?;
    Ok(())
}
