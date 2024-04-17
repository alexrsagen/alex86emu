pub mod error;
use std::collections::HashMap;

use error::Error;
use goblin::mach::cputype::{CPU_TYPE_X86, CPU_TYPE_X86_64};

use crate::cpu::Cpu;
use crate::mem::StackPointer;

use goblin::mach::load_command::{CommandVariant, LC_MAIN};
use goblin::mach::Mach;
use goblin::elf::program_header::PT_LOAD;
use goblin::elf::ProgramHeader;
use goblin::Object;
use iced_x86::{Code, Decoder, DecoderOptions};
use log::debug;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Execution {
    pub exit_code: u64,
    pub stdout: String,
    pub stderr: String,
}

fn handle_syscall(cpu: &mut Cpu, exit_code: &mut Option<u64>, stdout: &mut String, stderr: &mut String) -> Result<(), Error> {
    // syscall number:    rax
    // syscall arguments: rdi, rsi, rdx, r10, r8, r9
    match cpu.registers.rax {
        0x1 => { // 0x1 = write(fd = rdi, buf = rsi, count = rdx);
            let count = cpu.registers.rdx as usize;
            if count > cpu.stack.len() {
                return Err(crate::cpu::error::Error::StackOverflowRead.into());
            }
            let mut buf_ptr = StackPointer::new(cpu.registers.rsi, cpu.stack.len() as u64);
            let mut buf = Vec::new();
            for _ in 0..count {
                buf.push(cpu.stack[buf_ptr.value() as usize]);
                buf_ptr += 1;
            }
            let buf_str = String::from_utf8(buf)?;
            match cpu.registers.rdi {
                0x1 => {
                    debug!("SYSCALL: write(STDOUT, {:?})", &buf_str);
                    stdout.push_str(&buf_str);
                }
                0x2 => {
                    debug!("SYSCALL: write(STDERR, {:?})", &buf_str);
                    stderr.push_str(&buf_str);
                }
                fd => return Err(Error::UnimplementedFileDescriptor(fd)),
            }

            Ok(())
        }

        0x3c => { // 0x3c = exit(status = rdi)
            let status = cpu.registers.rdi;
            *exit_code = Some(status);
            debug!("SYSCALL: exit(0x{:x})", status);

            Ok(())
        }

        number => Err(Error::UnimplementedSyscall(number)),
    }
}

pub fn execute_from_x86_decoder(decoder: &mut Decoder) -> Result<Execution, Error> {
	let relative_instruction_pointer = decoder.ip() - decoder.position() as u64;

	// debug!("stack: {:02x?}", &self.stack[self.registers.rsp.0 as usize..]);
	// debug!("registers: {:?}", self.registers);

    let mut exit_code: Option<u64> = None;
	let mut stdout = String::new();
	let mut stderr = String::new();

    {
        let mut cpu = Cpu::new();
        // TODO: initialize cpu.registers.rbp to start of stack (end of memory)

        loop {
            // update instruction pointer
            let instruction = decoder.decode();
            if instruction.code() == Code::INVALID {
                return Err(Error::ProgramDidNotExit);
            }

            cpu.registers.rip = decoder.ip();

            if instruction.code() == Code::Syscall {
                handle_syscall(&mut cpu, &mut exit_code, &mut stdout, &mut stderr)?;
            } else if let Err(e) = cpu.execute_instruction(instruction) {
                return Err(Error::Cpu(e));
            }

            // check for program exit via syscall
            if exit_code.is_some() {
                break;
            }

            if cpu.registers.rip != decoder.ip() {
                // instruction pointer modified by program
                // update instruction pointer
                let rip_position = cpu.registers.rip.checked_sub(relative_instruction_pointer).ok_or(Error::UnimplementedInstructionPointerOutsideProgramSpace)? as usize;
                decoder.set_ip(cpu.registers.rip);
                decoder.set_position(rip_position).map_err(|_e| Error::UnimplementedInstructionPointerOutsideProgramSpace)?;
            }

            // debug!("stack: {:02x?}", &cpu.stack[cpu.registers.rsp.0 as usize..]);
            // debug!("registers: {:?}", cpu.registers);
        }
    }

    if let Some(exit_code) = exit_code {
        Ok(Execution {
            exit_code,
            stdout,
            stderr,
        })
    } else {
        Err(Error::ProgramDidNotExit)
    }
}

pub fn execute_from_binary_slice(binary: &[u8]) -> Result<Execution, Error> {
    match Object::parse(&binary) {
        Ok(Object::Elf(elf)) => {
            let bitness = if elf.is_64 {
                64
            } else {
                32
            };

            let mut load_headers: Vec<&ProgramHeader> = elf.program_headers.iter().filter(|phdr| phdr.p_type == PT_LOAD).collect();
            load_headers.sort_by(|a, b| a.p_vaddr.cmp(&b.p_vaddr));

            if let Some(load_base_header) = load_headers.first() {
                let relative_instruction_pointer = load_base_header.p_vaddr;
                let mut decoder = Decoder::with_ip(bitness, &binary, relative_instruction_pointer, DecoderOptions::NONE);

                let entry_point_rva = elf.header.e_entry;
                let entry_point_addr = entry_point_rva.saturating_sub(relative_instruction_pointer) as usize;
                decoder.set_position(entry_point_addr)?;
                decoder.set_ip(entry_point_rva);

                let execution = execute_from_x86_decoder(&mut decoder)?;
                Ok(execution)
            } else {
                Err(Error::ElfLoadHeaderMissing)
            }
        }

        Ok(Object::PE(pe)) => {
            let bitness = if pe.is_64 {
                64
            } else {
                32
            };

            if let Some(optional_header) = pe.header.optional_header {
                let relative_instruction_pointer = optional_header.windows_fields.image_base;
                let entry_point_rva = optional_header.standard_fields.address_of_entry_point;
                let entry_point_addr = entry_point_rva.saturating_sub(relative_instruction_pointer) as usize;

                let mut decoder = Decoder::with_ip(bitness, &binary, entry_point_rva, DecoderOptions::NONE);
                decoder.set_position(entry_point_addr)?;
                // decoder.set_ip(entry_point_rva);

                let execution = execute_from_x86_decoder(&mut decoder)?;
                Ok(execution)
            } else {
                Err(Error::PeOptionalHeaderMissing)
            }
        }

        Ok(Object::Mach(Mach::Binary(mach_o))) => {
            let bitness = if mach_o.is_64 {
                64
            } else {
                32
            };

            if let Some(CommandVariant::Main(main_load_command)) = mach_o.load_commands.into_iter().map(|cmd| cmd.command).find(|cmd| cmd.cmd() == LC_MAIN) {
                let entry_point_rva = mach_o.entry;
                let entry_point_addr = main_load_command.entryoff as usize;

                let mut decoder = Decoder::with_ip(bitness, &binary, entry_point_rva, DecoderOptions::NONE);
                decoder.set_position(entry_point_addr)?;
                // decoder.set_ip(entry_point_rva);

                let execution = execute_from_x86_decoder(&mut decoder)?;
                Ok(execution)
            } else {
                Err(Error::MachOLoadCommandMissing)
            }
        }

        Ok(Object::Mach(Mach::Fat(mach_fat))) => {
            if let Ok(Some(fat_arch)) = mach_fat.find_cputype(CPU_TYPE_X86).or(mach_fat.find_cputype(CPU_TYPE_X86_64)) {
                let start = fat_arch.offset as usize;
                if let Some(end) = start.checked_add(fat_arch.size as usize) {
                    if start >= binary.len() {
                        return Err(Error::MachFatNoX86);
                    }
                    if end > binary.len() {
                        return Err(Error::MachFatNoX86);
                    }

                    execute_from_binary_slice(&binary[start..end])
                } else {
                    Err(Error::MachFatNoX86)
                }
            } else {
                Err(Error::MachFatNoX86)
            }
        }

        Ok(_) => Err(Error::UnimplementedBinaryFileFormat),
        Err(e) => Err(e.into()),
    }
}

pub fn get_exports_from_binary_slice(binary: &[u8]) -> Result<HashMap<String, u64>, Error> {
    match Object::parse(&binary) {
        Ok(Object::Elf(elf)) => {
            let mut label_addresses = HashMap::new();
            for symbol in &elf.syms {
                if symbol.st_value == 0 {
                    continue
                }
                if let Some(symbol_name) = elf.strtab.get_at(symbol.st_name) {
                    if symbol_name == "" {
                        continue
                    }
                    label_addresses.insert(symbol_name.to_owned(), symbol.st_value);
                }
            }
            Ok(label_addresses)
        }

        Ok(Object::PE(pe)) => {
            let mut label_addresses = HashMap::new();
            for export in pe.exports {
                if let Some(name) = export.name {
                    label_addresses.insert(name.to_owned(), export.rva as u64);
                }
            }
            Ok(label_addresses)
        }

        Ok(Object::Mach(Mach::Binary(mach_o))) => {
            let mut label_addresses = HashMap::new();
            for export in mach_o.exports()? {
                match export.info {
                    goblin::mach::exports::ExportInfo::Regular { address, .. } => {
                        label_addresses.insert(export.name, address);
                    }
                    _ => {}
                }
            }
            Ok(label_addresses)
        }

        Ok(Object::Mach(Mach::Fat(mach_fat))) => {
            if let Ok(Some(fat_arch)) = mach_fat.find_cputype(CPU_TYPE_X86).or(mach_fat.find_cputype(CPU_TYPE_X86_64)) {
                let start = fat_arch.offset as usize;
                if let Some(end) = start.checked_add(fat_arch.size as usize) {
                    if start >= binary.len() {
                        return Err(Error::MachFatNoX86);
                    }
                    if end > binary.len() {
                        return Err(Error::MachFatNoX86);
                    }

                    get_exports_from_binary_slice(&binary[start..end])
                } else {
                    Err(Error::MachFatNoX86)
                }
            } else {
                Err(Error::MachFatNoX86)
            }
        }

        Ok(_) => Err(Error::UnimplementedBinaryFileFormat),
        Err(e) => Err(e.into()),
    }
}