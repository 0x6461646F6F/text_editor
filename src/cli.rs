use std::{fmt::Display, path::PathBuf};

pub enum Command {
    Help,
    Edit(Option<PathBuf>),
}

#[derive(Debug, Clone)]
pub enum ParseError {
    UnknownOption(String),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnknownOption(s) => write!(f, "unknown option: {s}"),
        }
    }
}

type CommandResult = Result<Command, ParseError>;

pub fn parse() -> CommandResult {
    let mut args = std::env::args_os().skip(1);

    while let Some(arg) = args.next() {
        if arg == "-h" || arg == "--help" {
            return Ok(Command::Help);
        }

        let looks_like_flag = arg.to_str().is_some_and(|s| s.starts_with('-'));
        if looks_like_flag {
            return Err(ParseError::UnknownOption(arg.to_string_lossy().to_string()));
        }

        return Ok(Command::Edit(Some(PathBuf::from(arg))));
    }

    Ok(Command::Edit(None))
}

pub fn print_help() {
    println!("usage: editor [-h | --help] [<path>]");
    println!();
    println!("arguments:");
    println!("  <path>        path to a file open, or none for an empty buffer");
    println!();
    println!("options:");
    println!("  -h, --help    print this message");
}

pub fn report_error(e: ParseError) {
    eprintln!("{e}");
    eprintln!("usage: editor [-h | --help] [<path>]");
}
