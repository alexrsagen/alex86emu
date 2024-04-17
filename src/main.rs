mod alloc;
mod cpu;
mod mem;
mod program;
mod args;
mod logger;

use std::path::Path;

use anyhow::{anyhow, Context, Result};
use base64::prelude::*;
use log::info;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;

#[global_allocator]
static GLOBAL: alloc::SystemTrackingAllocator = alloc::SystemTrackingAllocator::new_system();

#[derive(Debug, Default)]
struct BinaryInfo {
    exit_code: u64,
    win_function_address: u64,
    program_output: String,
}

fn get_binary_info(binary: &[u8]) -> Result<BinaryInfo> {
    let mut binary_info = BinaryInfo::default();

    let execution = program::execute_from_binary_slice(binary)?;
    binary_info.exit_code = execution.exit_code;
    binary_info.program_output = execution.stdout;

    // [?] What is the address of the "win_function" symbol in the binary?
    let exports = program::get_exports_from_binary_slice(binary)?;
    binary_info.win_function_address = *exports.get("win_function").ok_or(anyhow!("unable to get win_function address from binary"))?;

    Ok(binary_info)
}

#[derive(Debug)]
enum State {
    AwaitingBinaryReadPrompt,
    AwaitingBinary,
    AwaitingQuestionOne(BinaryInfo),
    AwaitingQuestionTwo(BinaryInfo),
    Complete,
}

async fn emulate_file(binary_path: &Path) -> Result<()> {
    let binary = tokio::fs::read(binary_path).await?;
    let execution = program::execute_from_binary_slice(&binary)?;

    info!("program exited with code {}", execution.exit_code);
    info!("stdout: {:?}", execution.stdout);
    info!("stderr: {:?}", execution.stderr);

    let exports = program::get_exports_from_binary_slice(&binary)?;
    if let Some(win_function_address) = exports.get("win_function") {
        info!("address of export 'win_function': 0x{:x}", win_function_address);
    }

    Ok(())
}

async fn complete_challenge(remote_address: &str) -> Result<()> {
    let mut stream = TcpStream::connect(remote_address).await?;
    let (reader, writer) = stream.split();
    let mut writer = BufWriter::with_capacity(1024, writer);

    let mut state = State::AwaitingBinaryReadPrompt;
    let mut buf = Vec::with_capacity(8192);
    loop {
        match reader.try_read_buf(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                match state {
                    State::AwaitingBinaryReadPrompt => {
                        if buf.ends_with(b"[?] Are you ready to receive the binary? [yes/no] ") {
                            writer.write_all(b"yes\n").await?;
                            writer.flush().await?;
                            buf.clear();
                            state = State::AwaitingBinary;

                            info!("[?] Are you ready to receive the binary? [yes/no] > yes");
                        }
                    }
                    State::AwaitingBinary => {
                        let prefix = b"[*] \n    Base64 ELF: ";
                        if let Some(start) = memchr::memmem::find_iter(&buf, prefix).next() {
                            let (_left, right) = buf.split_at(start + prefix.len());
                            if let Some(end) = memchr::memchr(b'\n', right) {
                                let (binary_base64, _right) = right.split_at(end);
                                let binary = BASE64_STANDARD.decode(&binary_base64)?;
                                let binary_info = get_binary_info(&binary).context("unable to parse/execute binary. make sure that it is a valid ELF64 binary that only utilizes the stack (no memory), contains a 'win_function' label and that executing the entry point outputs a string to STDOUT using the write syscall.")?;

                                info!("[*] Binary received and executed. Exit code {}", &binary_info.exit_code);
                                state = State::AwaitingQuestionOne(binary_info);
                            }
                        }
                    }
                    State::AwaitingQuestionOne(binary_info) => {
                        if buf.ends_with(b"[?] What is the output from running the binary?\n> ") {
                            writer.write_all(binary_info.program_output.as_bytes()).await?;
                            writer.write_u8(b'\n').await?;
                            writer.flush().await?;
                            buf.clear();

                            info!("[?] What is the output from running the binary? > {}", &binary_info.program_output);
                            state = State::AwaitingQuestionTwo(binary_info);
                        } else {
                            state = State::AwaitingQuestionOne(binary_info);
                        }
                    }
                    State::AwaitingQuestionTwo(binary_info) => {
                        if buf.ends_with(b"[?] What is the address of win_function in hex?\n> ") {
                            let win_function_address_hex = format!("0x{:x}", binary_info.win_function_address);
                            writer.write_all(win_function_address_hex.as_bytes()).await?;
                            writer.write_u8(b'\n').await?;
                            writer.flush().await?;
                            buf.clear();

                            info!("[?] What is the address of win_function in hex? > {}", &win_function_address_hex);
                            state = State::Complete;
                        } else {
                            state = State::AwaitingQuestionTwo(binary_info);
                        }
                    }
                    State::Complete => {}
                }
            }
            Err(ref e) if e.kind() == tokio::io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                return Err(e.into());
            }
        }
    }

    let response_string = String::from_utf8(buf)?;
    info!("{}", response_string);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // parse command-line arguments
    let args = args::Args::parse();

    // set up logger
    logger::try_init(args.log_level)?;

    match args.command {
        args::Command::EmulateFile { binary_path } => emulate_file(&binary_path).await,
        args::Command::CompleteChallenge { remote_address } => complete_challenge(&remote_address).await,
    }
}
