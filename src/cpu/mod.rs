pub mod error;
pub mod registers;

use crate::mem::StackPointer;

use registers::Registers;
use error::Error;

use iced_x86::{Code, Instruction, OpKind, Register};
// use log::debug;

// TODO: implement memory, access stack via rbp - rsp
const STACK_SIZE: usize = 1024; // 1 KiB

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cpu {
    pub stack: Box<[u8; STACK_SIZE]>,
    pub registers: Registers,
}

impl Default for Cpu {
    fn default() -> Self {
        // initialize stack
        let mut stack = Vec::with_capacity(STACK_SIZE);
        stack.resize(stack.capacity(), 0);
        let stack = stack.into_boxed_slice().try_into().unwrap();

        // initialize stack pointer
        let mut registers = Registers::default();
        registers.rsp = StackPointer::new(0, STACK_SIZE as u64);

        Self {
            stack,
            registers,
        }
    }
}

impl Cpu {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn execute_instruction(&mut self, instruction: Instruction) -> Result<(), Error> {
        match instruction.code() {
            Code::Mov_r64_imm64 => self.set_register(instruction.op0_register(), instruction.immediate64()),

            Code::Mov_rm64_r64 => {
                match instruction.op0_kind() {
                    OpKind::Register => self.set_register(instruction.op0_register(), self.get_register_u64(instruction.op1_register())?),
                    _ => Err(Error::UnimplementedInstruction(instruction))
                }
            }

            Code::Push_r64 => self.push_stack_register(instruction.op0_register()),
            Code::Pushq_imm8 => self.push_stack_value(instruction.immediate8().to_le_bytes().as_slice()),
            Code::Pop_r64 => self.pop_stack_register(instruction.op0_register()),

            Code::Jmp_rel8_64 => {
                self.registers.rip = instruction.near_branch64();
                Ok(())
            }

            Code::Xor_rm32_r32 => {
                // TODO: truncate data to 32 bits before performing arithmetic?
                match instruction.op0_kind() {
                    OpKind::Register => self.set_register(instruction.op0_register(), self.get_register_u64(instruction.op0_register())? ^ self.get_register_u64(instruction.op1_register())?),
                    _ => Err(Error::UnimplementedInstruction(instruction))
                }
            }

            _ => Err(Error::UnimplementedInstruction(instruction))
        }
    }

    fn push_stack_register(&mut self, register: Register) -> Result<(), Error> {
        let mut value = self.get_register_u64(register)?.to_le_bytes().to_vec();
        value.resize(register.size(), 0);
        self.push_stack_value(&value)
    }

    fn push_stack_value(&mut self, value: &[u8]) -> Result<(), Error> {
        if value.len() > self.stack.len() {
            return Err(Error::StackOverflowWrite);
        }
        let alignment = if value.len() % 8 != 0 {
            8 - (value.len() % 8) as u64
        } else {
            0
        };

        // debug!("push size: {} b / alignment size: {} b", value.len(), alignment);
        // debug!("push value: {:02x?}", &value);
        // debug!("rsp / stack before push: {} (0x{:x}) / {:02x?}", self.registers.rsp.0, self.registers.rsp.0, &self.stack[self.registers.rsp.0 as usize..]);

        self.registers.rsp -= alignment;
        for x in value.iter().rev() {
            self.registers.rsp -= 1;
            self.stack[self.registers.rsp.value() as usize] = *x;
        }

        // debug!("rsp / stack after push: {} (0x{:x}) / {:02x?}", self.registers.rsp.0, self.registers.rsp.0, &self.stack[self.registers.rsp.0 as usize..]);

        Ok(())
    }

    fn pop_stack_value(&mut self, size: usize) -> Result<Vec<u8>, Error> {
        if size > self.stack.len() {
            return Err(Error::StackOverflowRead);
        }
        let alignment = if size % 8 != 0 {
            8 - (size % 8) as u64
        } else {
            0
        };

        // debug!("push size: {} b / alignment size: {} b", size, alignment);
        // debug!("rsp / stack before pop: {} (0x{:x}) / {:02x?}", self.registers.rsp.0, self.registers.rsp.0, &self.stack[self.registers.rsp.0 as usize..]);

        let mut value = Vec::new();
        for _ in 0..size {
            value.push(self.stack[self.registers.rsp.value() as usize]);
            self.registers.rsp += 1;
        }
        self.registers.rsp += alignment;

        // debug!("rsp / stack after pop: {} (0x{:x}) / {:02x?}", self.registers.rsp.0, self.registers.rsp.0, &self.stack[self.registers.rsp.0 as usize..]);
        // debug!("pop value: {:02x?}", &value);

        Ok(value)
    }

    fn pop_stack_register(&mut self, register: Register) -> Result<(), Error> {
        let bytes = self.pop_stack_value(register.size())?;
        let size = bytes.len();
        let value = match size {
            1 => u8::from_le_bytes(bytes.try_into().map_err(|_e| Error::UnimplementedRegisterSize(size))?) as u64,
            2 => u16::from_le_bytes(bytes.try_into().map_err(|_e| Error::UnimplementedRegisterSize(size))?) as u64,
            4 => u32::from_le_bytes(bytes.try_into().map_err(|_e| Error::UnimplementedRegisterSize(size))?) as u64,
            8 => u64::from_le_bytes(bytes.try_into().map_err(|_e| Error::UnimplementedRegisterSize(size))?),
            size => return Err(Error::UnimplementedRegisterSize(size)),
        };
        self.set_register(register, value)
    }

    fn get_register_u64(&self, register: Register) -> Result<u64, Error> {
        match register {
            Register::AL | Register::AH | Register::AX | Register::EAX | Register::RAX => Ok(self.registers.rax),
            Register::CL | Register::CH | Register::CX | Register::ECX | Register::RCX => Ok(self.registers.rcx),
            Register::DL | Register::DH | Register::DX | Register::EDX | Register::RDX => Ok(self.registers.rdx),
            Register::BL | Register::BH | Register::BX | Register::EBX | Register::RBX => Ok(self.registers.rbx),
            Register::SPL | Register::SP | Register::ESP | Register::RSP => Ok(self.registers.rsp.into()),
            Register::BPL | Register::BP | Register::EBP | Register::RBP => Ok(self.registers.rbp),
            Register::SIL | Register::SI | Register::ESI | Register::RSI => Ok(self.registers.rsi),
            Register::DIL | Register::DI | Register::EDI | Register::RDI => Ok(self.registers.rdi),
            Register::R8L | Register::R8W | Register::R8D | Register::R8 => Ok(self.registers.r8),
            Register::R9L | Register::R9W | Register::R9D | Register::R9 => Ok(self.registers.r9),
            Register::R10L | Register::R10W | Register::R10D | Register::R10 => Ok(self.registers.r10),
            Register::R11L | Register::R11W | Register::R11D | Register::R11 => Ok(self.registers.r11),
            Register::R12L | Register::R12W | Register::R12D | Register::R12 => Ok(self.registers.r12),
            Register::R13L | Register::R13W | Register::R13D | Register::R13 => Ok(self.registers.r13),
            Register::R14L | Register::R14W | Register::R14D | Register::R14 => Ok(self.registers.r14),
            Register::R15L | Register::R15W | Register::R15D | Register::R15 => Ok(self.registers.r15),
            Register::EIP | Register::RIP => Ok(self.registers.rip),
            Register::ES => Ok(self.registers.es as u64),
            Register::CS => Ok(self.registers.cs as u64),
            Register::SS => Ok(self.registers.ss as u64),
            Register::DS => Ok(self.registers.ds as u64),
            Register::FS => Ok(self.registers.fs as u64),
            Register::GS => Ok(self.registers.gs as u64),
            Register::CR0 => Ok(self.registers.cr0),
            Register::CR1 => Ok(self.registers.cr1),
            Register::CR2 => Ok(self.registers.cr2),
            Register::CR3 => Ok(self.registers.cr3),
            Register::CR4 => Ok(self.registers.cr4),
            Register::CR5 => Ok(self.registers.cr5),
            Register::CR6 => Ok(self.registers.cr6),
            Register::CR7 => Ok(self.registers.cr7),
            Register::CR8 => Ok(self.registers.cr8),
            Register::CR9 => Ok(self.registers.cr9),
            Register::CR10 => Ok(self.registers.cr10),
            Register::CR11 => Ok(self.registers.cr11),
            Register::CR12 => Ok(self.registers.cr12),
            Register::CR13 => Ok(self.registers.cr13),
            Register::CR14 => Ok(self.registers.cr14),
            Register::CR15 => Ok(self.registers.cr15),
            Register::DR0 => Ok(self.registers.dr0),
            Register::DR1 => Ok(self.registers.dr1),
            Register::DR2 => Ok(self.registers.dr2),
            Register::DR3 => Ok(self.registers.dr3),
            Register::DR4 => Ok(self.registers.dr4),
            Register::DR5 => Ok(self.registers.dr5),
            Register::DR6 => Ok(self.registers.dr6),
            Register::DR7 => Ok(self.registers.dr7),
            Register::DR8 => Ok(self.registers.dr8),
            Register::DR9 => Ok(self.registers.dr9),
            Register::DR10 => Ok(self.registers.dr10),
            Register::DR11 => Ok(self.registers.dr11),
            Register::DR12 => Ok(self.registers.dr12),
            Register::DR13 => Ok(self.registers.dr13),
            Register::DR14 => Ok(self.registers.dr14),
            Register::DR15 => Ok(self.registers.dr15),
            Register::TR0 => Ok(self.registers.tr0 as u64),
            Register::TR1 => Ok(self.registers.tr1 as u64),
            Register::TR2 => Ok(self.registers.tr2 as u64),
            Register::TR3 => Ok(self.registers.tr3 as u64),
            Register::TR4 => Ok(self.registers.tr4 as u64),
            Register::TR5 => Ok(self.registers.tr5 as u64),
            Register::TR6 => Ok(self.registers.tr6 as u64),
            Register::TR7 => Ok(self.registers.tr7 as u64),
            register => Err(Error::UnimplementedRegister(register)),
        }
    }

    fn set_register(&mut self, register: Register, value: u64) -> Result<(), Error> {
        match register {
            Register::AL | Register::AH | Register::AX | Register::EAX | Register::RAX => { self.registers.rax = value; },
            Register::CL | Register::CH | Register::CX | Register::ECX | Register::RCX => { self.registers.rcx = value; },
            Register::DL | Register::DH | Register::DX | Register::EDX | Register::RDX => { self.registers.rdx = value; },
            Register::BL | Register::BH | Register::BX | Register::EBX | Register::RBX => { self.registers.rbx = value; },
            Register::SPL | Register::SP | Register::ESP | Register::RSP => { self.registers.rsp = StackPointer::new(value, STACK_SIZE as u64); },
            Register::BPL | Register::BP | Register::EBP | Register::RBP => { self.registers.rbp = value; },
            Register::SIL | Register::SI | Register::ESI | Register::RSI => { self.registers.rsi = value; },
            Register::DIL | Register::DI | Register::EDI | Register::RDI => { self.registers.rdi = value; },
            Register::R8L | Register::R8W | Register::R8D | Register::R8 => { self.registers.r8 = value; },
            Register::R9L | Register::R9W | Register::R9D | Register::R9 => { self.registers.r9 = value; },
            Register::R10L | Register::R10W | Register::R10D | Register::R10 => { self.registers.r10 = value; },
            Register::R11L | Register::R11W | Register::R11D | Register::R11 => { self.registers.r11 = value; },
            Register::R12L | Register::R12W | Register::R12D | Register::R12 => { self.registers.r12 = value; },
            Register::R13L | Register::R13W | Register::R13D | Register::R13 => { self.registers.r13 = value; },
            Register::R14L | Register::R14W | Register::R14D | Register::R14 => { self.registers.r14 = value; },
            Register::R15L | Register::R15W | Register::R15D | Register::R15 => { self.registers.r15 = value; },
            Register::EIP | Register::RIP => { self.registers.rip = value; },
            Register::ES => { self.registers.es = value as u16; },
            Register::CS => { self.registers.cs = value as u16; },
            Register::SS => { self.registers.ss = value as u16; },
            Register::DS => { self.registers.ds = value as u16; },
            Register::FS => { self.registers.fs = value as u16; },
            Register::GS => { self.registers.gs = value as u16; },
            Register::CR0 => { self.registers.cr0 = value; },
            Register::CR1 => { self.registers.cr1 = value; },
            Register::CR2 => { self.registers.cr2 = value; },
            Register::CR3 => { self.registers.cr3 = value; },
            Register::CR4 => { self.registers.cr4 = value; },
            Register::CR5 => { self.registers.cr5 = value; },
            Register::CR6 => { self.registers.cr6 = value; },
            Register::CR7 => { self.registers.cr7 = value; },
            Register::CR8 => { self.registers.cr8 = value; },
            Register::CR9 => { self.registers.cr9 = value; },
            Register::CR10 => { self.registers.cr10 = value; },
            Register::CR11 => { self.registers.cr11 = value; },
            Register::CR12 => { self.registers.cr12 = value; },
            Register::CR13 => { self.registers.cr13 = value; },
            Register::CR14 => { self.registers.cr14 = value; },
            Register::CR15 => { self.registers.cr15 = value; },
            Register::DR0 => { self.registers.dr0 = value; },
            Register::DR1 => { self.registers.dr1 = value; },
            Register::DR2 => { self.registers.dr2 = value; },
            Register::DR3 => { self.registers.dr3 = value; },
            Register::DR4 => { self.registers.dr4 = value; },
            Register::DR5 => { self.registers.dr5 = value; },
            Register::DR6 => { self.registers.dr6 = value; },
            Register::DR7 => { self.registers.dr7 = value; },
            Register::DR8 => { self.registers.dr8 = value; },
            Register::DR9 => { self.registers.dr9 = value; },
            Register::DR10 => { self.registers.dr10 = value; },
            Register::DR11 => { self.registers.dr11 = value; },
            Register::DR12 => { self.registers.dr12 = value; },
            Register::DR13 => { self.registers.dr13 = value; },
            Register::DR14 => { self.registers.dr14 = value; },
            Register::DR15 => { self.registers.dr15 = value; },
            Register::TR0 => { self.registers.tr0 = value as u32; },
            Register::TR1 => { self.registers.tr1 = value as u32; },
            Register::TR2 => { self.registers.tr2 = value as u32; },
            Register::TR3 => { self.registers.tr3 = value as u32; },
            Register::TR4 => { self.registers.tr4 = value as u32; },
            Register::TR5 => { self.registers.tr5 = value as u32; },
            Register::TR6 => { self.registers.tr6 = value as u32; },
            Register::TR7 => { self.registers.tr7 = value as u32; },
            register => return Err(Error::UnimplementedRegister(register)),
        }
        Ok(())
    }
}