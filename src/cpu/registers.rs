use crate::mem::StackPointer;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Registers {
    /// register instruction pointer
    pub rip: u64,
    /// register a extended
    pub rax: u64,
    /// register b extended
    pub rbx: u64,
    /// register c extended
    pub rcx: u64,
    /// register d extended
    pub rdx: u64,
    /// register base pointer (start of stack)
    pub rbp: u64,
    /// register stack pointer (current location in stack, growing downwards)
    pub rsp: StackPointer,
    /// register source index (source for data copies)
    pub rsi: u64,
    /// register destination index (destination for data copies)
    pub rdi: u64,
    /// register 8
    pub r8: u64,
    /// register 9
    pub r9: u64,
    /// register 10
    pub r10: u64,
    /// register 11
    pub r11: u64,
    /// register 12
    pub r12: u64,
    /// register 13
    pub r13: u64,
    /// register 14
    pub r14: u64,
    /// register 15
    pub r15: u64,
    /// code segment
    pub cs: u16,
    /// data segment
    pub ds: u16,
    /// stack segment
    pub ss: u16,
    /// extra segment
    pub es: u16,
    /// general-purpose segment
    pub fs: u16,
    /// general-purpose segment
    pub gs: u16,
    // register flags
    pub rflags: u64,
    /// control register 0
    pub cr0: u64,
    /// control register 1
    pub cr1: u64,
    /// control register 2
    pub cr2: u64,
    /// control register 3
    pub cr3: u64,
    /// control register 4
    pub cr4: u64,
    /// control register 5
    pub cr5: u64,
    /// control register 6
    pub cr6: u64,
    /// control register 7
    pub cr7: u64,
    /// control register 8
    pub cr8: u64,
    /// control register 9
    pub cr9: u64,
    /// control register 10
    pub cr10: u64,
    /// control register 11
    pub cr11: u64,
    /// control register 12
    pub cr12: u64,
    /// control register 13
    pub cr13: u64,
    /// control register 14
    pub cr14: u64,
    /// control register 15
    pub cr15: u64,
    /// debug register 0
    pub dr0: u64,
    /// debug register 1
    pub dr1: u64,
    /// debug register 2
    pub dr2: u64,
    /// debug register 3
    pub dr3: u64,
    /// debug register 4
    pub dr4: u64,
    /// debug register 5
    pub dr5: u64,
    /// debug register 6
    pub dr6: u64,
    /// debug register 7
    pub dr7: u64,
    /// debug register 8
    pub dr8: u64,
    /// debug register 9
    pub dr9: u64,
    /// debug register 10
    pub dr10: u64,
    /// debug register 11
    pub dr11: u64,
    /// debug register 12
    pub dr12: u64,
    /// debug register 13
    pub dr13: u64,
    /// debug register 14
    pub dr14: u64,
    /// debug register 15
    pub dr15: u64,
    /// test register 0
    pub tr0: u32,
    /// test register 1
    pub tr1: u32,
    /// test register 2
    pub tr2: u32,
    /// test register 3
    pub tr3: u32,
    /// test register 4
    pub tr4: u32,
    /// test register 5
    pub tr5: u32,
    /// test register 6
    pub tr6: u32,
    /// test register 7
    pub tr7: u32,
}

impl Registers {
    #[allow(unused)]
    pub fn new() -> Self {
        Self::default()
    }
}