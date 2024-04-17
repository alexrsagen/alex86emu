use iced_x86::{Instruction, Register};

#[derive(Debug)]
pub enum Error {
    UnimplementedRegister(Register),
    UnimplementedRegisterSize(usize),
    UnimplementedInstruction(Instruction),
    StackOverflowRead,
    StackOverflowWrite,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnimplementedRegister(register) => write!(f, "register {:?} is not implemented", register),
            Self::UnimplementedRegisterSize(size) => write!(f, "register with size {} is not implemented", size),
            Self::UnimplementedInstruction(instruction) => write!(f, "opcode {:?} is not implemented (in instruction {})", instruction.code(), instruction),
            Self::StackOverflowRead => write!(f, "attempted to read a value larger than the stack from the stack"),
            Self::StackOverflowWrite => write!(f, "attempted to write a value larger than the stack to the stack"),
        }
    }
}
