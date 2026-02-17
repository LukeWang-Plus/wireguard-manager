//! Entry point: dispatches to CLI or TUI based on command-line arguments.

mod cli;
mod key;
mod manager;
mod tui;

#[cfg(test)]
mod tests;

use clap::Parser;
use cli::Cli;

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(cmd) => cli::run(cmd),
        None => tui::run(),
    }
}
