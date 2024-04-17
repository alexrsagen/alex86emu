mod alloc;
mod cpu;
mod mem;
mod program;
mod args;
mod logger;

use anyhow::Result;
use log::info;

#[global_allocator]
static GLOBAL: alloc::SystemTrackingAllocator = alloc::SystemTrackingAllocator::new_system();

#[tokio::main]
async fn main() -> Result<()> {
    // parse command-line arguments
    let args = args::Args::parse();

    // set up logger
    logger::try_init(args.log_level)?;

    // read binary file
    let binary = tokio::fs::read(args.binary_path).await?;
    let execution = program::execute_from_binary_slice(&binary)?;
    info!("program exited with code {}", execution.exit_code);
    info!("stdout: {:?}", execution.stdout);
    info!("stderr: {:?}", execution.stderr);

    Ok(())
}
