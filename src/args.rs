use clap::Parser;

#[derive(Debug, Parser)]
#[clap(author, about, version)]
pub struct Args {
    /// Log level [off|error|warn|info|debug|trace]
    #[clap(long, short = 'l', default_value = "info")]
    pub log_level: log::LevelFilter,

    /// Input binary file path
    #[clap(index = 1)]
    pub binary_path: std::path::PathBuf,
}

impl Args {
    pub fn parse() -> Self {
        Parser::parse()
    }
}
