#[derive(Debug)]
pub enum Error {
    Cpu(crate::cpu::error::Error),
    Goblin(goblin::error::Error),
    Iced(iced_x86::IcedError),
    FromUtf8Error(std::string::FromUtf8Error),
    UnimplementedInstructionPointerOutsideProgramSpace,
    UnimplementedSyscall(u64),
    UnimplementedFileDescriptor(u64),
    UnimplementedBinaryFileFormat,
    ProgramDidNotExit,
    ElfLoadHeaderMissing,
    PeOptionalHeaderMissing,
    MachOLoadCommandMissing,
    MachFatNoX86,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cpu(e) => write!(f, "CPU error: {}", e),
            Self::Goblin(e) => write!(f, "error parsing binary program file: {}", e),
            Self::Iced(e) => write!(f, "error parsing program code: {}", e),
            Self::FromUtf8Error(e) => write!(f, "error during write syscall: invalid utf-8 string: {}", e),
            Self::UnimplementedInstructionPointerOutsideProgramSpace => write!(f, "instruction pointer changed by program and is no longer within the program space - memory is not implemented"),
            Self::UnimplementedSyscall(number) => write!(f, "syscall {} (0x{:x}) is not implemented", number, number),
            Self::UnimplementedFileDescriptor(fd) => write!(f, "file descriptor {} (0x{:x}) is not implemented", fd, fd),
            Self::UnimplementedBinaryFileFormat => write!(f, "unimplemented binary file format"),
            Self::ProgramDidNotExit => write!(f, "program did not exit in a clean manner"),
            Self::ElfLoadHeaderMissing => write!(f, "unable to find ELF load header"),
            Self::PeOptionalHeaderMissing => write!(f, "unable to find PE optional header"),
            Self::MachOLoadCommandMissing => write!(f, "unable to find Mach-O load command LC_MAIN"),
            Self::MachFatNoX86 => write!(f, "unable to find an x86 binary in fat Mach binary"),
        }
    }
}

impl From<crate::cpu::error::Error> for Error {
    fn from(e: crate::cpu::error::Error) -> Self {
        Self::Cpu(e)
    }
}

impl From<goblin::error::Error> for Error {
    fn from(e: goblin::error::Error) -> Self {
        Self::Goblin(e)
    }
}

impl From<iced_x86::IcedError> for Error {
    fn from(e: iced_x86::IcedError) -> Self {
        Self::Iced(e)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Self::FromUtf8Error(e)
    }
}
