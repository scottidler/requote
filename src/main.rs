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
}

fn main() -> Result<()> {
    let args = Args::parse();
    let path = Path::new(&args.path);
    process_path(path, args.recursive, args.reverse)?;
    Ok(())
}

fn process_path(path: &Path, recursive: bool, reverse: bool) -> Result<()> {
    if path.is_dir() && recursive {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                process_path(&path, recursive, reverse)?;
            } else if path.is_file() {
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
    // Parser for content inside quotes
    let content_inside_quotes = filter(|&c: &char| c != '\'' && c != '\"')
        .repeated()
        .collect::<String>();

    // Triple quotes parser
    let triple_quote_parser = |quote: char, opposite_quote: char| {
        just::<_, [char; 3], Simple<char>>([quote, quote, quote])
            .ignore_then(
                filter(move |&c| c != opposite_quote)
                    .repeated()
                    .collect::<String>()
            )
            .then_ignore(just::<_, [char; 3], Simple<char>>([quote, quote, quote]))
            .map(move |content: String| format!("{0}{1}{0}", if reverse { opposite_quote } else { quote }, content))
    };

    // Single and double quotes parser with content check
    let quote_parser = |quote: char, opposite_quote: char| {
        just::<_, char, Simple<char>>(quote)
            .ignore_then(content_inside_quotes.clone())
            .then_ignore(just::<_, char, Simple<char>>(quote))
            .map(move |content: String| {
                if content.contains(opposite_quote) {
                    format!("{0}{1}{0}", quote, content)
                } else {
                    format!("{0}{1}{0}", if reverse { quote } else { opposite_quote }, content)
                }
            })
    };

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

/*
fn process_content(content: &str, reverse: bool) -> String {
    // Correctly specify type annotations for just calls with Simple<char> as the error type
    let single_quote = just::<_, _, Simple<char>>('\'');
    let double_quote = just::<_, _, Simple<char>>('\"');

    let content_inside_quotes = filter(|&c: &char| c != '\'' && c != '\"')
        .repeated()
        .collect::<String>();

    // Triple quotes parser
    let triple_quote_parser = |quote: char, opposite_quote: char| {
        just::<_, _, Simple<char>>([quote, quote, quote])
            .ignore_then(
                filter(move |&c| c != opposite_quote)
                    .repeated()
                    .collect::<String>()
            )
            .then_ignore(just::<_, _, Simple<char>>([quote, quote, quote]))
            .map(move |content: String| format!("{0}{1}{0}", if reverse { opposite_quote } else { quote }, content))
    };

    // Single and double quotes parser with content check
    let quote_parser = |quote: char, opposite_quote: char| {
        just::<_, _, Simple<char>>(quote)
            .ignore_then(content_inside_quotes.clone())
            .then_ignore(just::<_, _, Simple<char>>(quote))
            .map(move |content: String| {
                if content.contains(opposite_quote) {
                    format!("{quote}{content}{quote}", quote = quote, content = content)
                } else {
                    format!("{opposite_quote}{content}{opposite_quote}", opposite_quote = opposite_quote, content = content)
                }
            })
    };

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

fn process_content(content: &str, reverse: bool) -> String {
    // Define basic parsers for single and double quotes
    let single_quote = just('\'');
    let double_quote = just('\"');

    // Parser for content inside quotes (excluding the quotes themselves)
    let content_inside_quotes = filter(|&c: &char| c != '\'' && c != '\"')
        .repeated()
        .collect::<String>();

    // Parser for triple-quoted strings
    let triple_quote_parser = |quote: char, opposite_quote: char| {
        just([quote, quote, quote])
            .ignore_then(
                filter(move |&c| c != opposite_quote)
                    .repeated()
                    .collect::<String>()
            )
            .then_ignore(just([quote, quote, quote]))
            .map(move |content: String| format!("{0}{1}{0}", if reverse { opposite_quote } else { quote }, content))
    };

    // Single and double quotes parser with content check
    let quote_parser = |quote: char, opposite_quote: char| {
        just(quote)
            .ignore_then(content_inside_quotes.clone())
            .then_ignore(just(quote))
            .map(move |content: String| {
                if content.contains(opposite_quote) {
                    format!("{quote}{content}{quote}")
                } else {
                    format!("{opposite_quote}{content}{opposite_quote}", quote = quote, opposite_quote = opposite_quote, content = content)
                }
            })
    };

    // Combine parsers to handle all cases
    let parser = choice((
        triple_quote_parser('\'', '\"'),
        triple_quote_parser('\"', '\''),
        quote_parser('\'', '\"'),
        quote_parser('\"', '\''),
        any().map(|c: char| c.to_string()),
    ))
    .repeated()
    .collect::<String>();

    // Parse the content
    parser.parse(content.chars().collect::<Vec<_>>())
          .unwrap_or_else(|e| {
              eprintln!("Error parsing content: {:?}", e);
              content.to_owned()
          })
}

fn process_content(content: &str, reverse: bool) -> String {
    // Parsers for triple quotes
    let triple_single_quote = just::<char, _, Simple<char>>("'''").to(String::new());
    let triple_double_quote = just::<char, _, Simple<char>>("\"\"\"").to(String::new());

    // Define parser for single and double quoted strings, checking content inside
    let single_quote = just('\'');
    let double_quote = just('\"');
    let content_inside_quotes = filter(|&c: &char| c != '\'' && c != '\"')
        .repeated()
        .collect::<String>()
        .then_ignore(end());

    // Enhanced parser for quoted strings to handle single-line and multi-line quotes
    let quoted_string = choice((
        triple_single_quote.map(|_| "\"\"\"".to_string()),
        triple_double_quote.map(|_| "'''".to_string()),
        single_quote.clone()
            .ignore_then(content_inside_quotes.clone())
            .then_ignore(single_quote)
            .map(|content| if reverse { format!("\"{}\"", content) } else { format!("'{}'", content) }),
        double_quote.clone()
            .ignore_then(content_inside_quotes.clone())
            .then_ignore(double_quote)
            .map(|content| {
                // Check if the content contains quotes that would conflict with the conversion
                if reverse && !content.contains('\'') {
                    format!("'{}'", content)
                } else {
                    // Default behavior for double-quoted strings without reverse logic
                    format!("\"{}\"", content)
                }
            }),
    ));

    // Parser for the whole content
    let parser = quoted_string
        .or(any().map(|c: char| c.to_string()))
        .repeated()
        .collect::<String>();

    parser.parse(content.chars().collect::<Vec<_>>())
          .unwrap_or_else(|e| {
              eprintln!("Error parsing content: {:?}", e);
              content.to_owned()
          })
}
*/
