use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Subcommand)]
pub enum Command {
    EmulateFile {
        /// Input binary file path
        binary_path: PathBuf,
    },
    CompleteChallenge {
        /// Remote address
        remote_address: String,
    },
}

#[derive(Debug, Parser)]
#[clap(author, about, version)]
pub struct Args {
    /// Log level [off|error|warn|info|debug|trace]
    #[clap(long, short = 'l', default_value = "info")]
    pub log_level: log::LevelFilter,

    #[clap(subcommand)]
    pub command: Command,
}

impl Args {
    pub fn parse() -> Self {
        Parser::parse()
    }
}