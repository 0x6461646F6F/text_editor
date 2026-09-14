mod app;
mod cli;
mod editor;
mod rope;
mod prompt;
mod message;

use std::{path::PathBuf, process::ExitCode};

fn main() -> ExitCode {
    match cli::parse() {
        Ok(cmd) => run_command(cmd),
        Err(e) => report_cli_error(e),
    }
}

fn run_command(cmd: cli::Command) -> ExitCode {
    match cmd {
        cli::Command::Help => {
            cli::print_help();
            ExitCode::SUCCESS
        }
        cli::Command::Edit(path) => run_editor(path),
    }
}

fn run_editor(path: Option<PathBuf>) -> ExitCode {
    match app::App::new(path.as_deref()).run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn report_cli_error(e: cli::ParseError) -> ExitCode {
    cli::report_error(e);
    ExitCode::from(2)
}
