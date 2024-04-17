#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct StackPointer {
    pointer: u64,
    stack_size: u64,
}

impl StackPointer {
    pub fn new(value: u64, stack_size: u64) -> Self {
        Self {
            pointer: value % stack_size,
            stack_size
        }
    }

    pub fn value(&self) -> u64 {
        self.pointer
    }

    pub fn set_wrapping(&mut self, value: u64) {
        self.pointer = value % self.stack_size;
    }
}

impl Into<u64> for StackPointer {
    fn into(self) -> u64 {
        self.pointer
    }
}

impl std::ops::Add<StackPointer> for StackPointer {
    type Output = StackPointer;

    fn add(self, x: StackPointer) -> Self::Output {
        StackPointer::new(self.pointer + x.pointer, self.stack_size)
    }
}

impl std::ops::AddAssign<StackPointer> for StackPointer {
    fn add_assign(&mut self, rhs: StackPointer) {
        *self = *self + rhs;
    }
}

impl std::ops::Add<u64> for StackPointer {
    type Output = StackPointer;

    fn add(self, x: u64) -> Self::Output {
        StackPointer::new(self.pointer + (x % self.stack_size), self.stack_size)
    }
}

impl std::ops::AddAssign<u64> for StackPointer {
    fn add_assign(&mut self, rhs: u64) {
        *self = *self + rhs;
    }
}

impl std::ops::Sub<StackPointer> for StackPointer {
    type Output = StackPointer;

    fn sub(self, x: StackPointer) -> Self::Output {
        StackPointer::new(self.pointer + x.pointer, self.stack_size)
    }
}

impl std::ops::SubAssign<StackPointer> for StackPointer {
    fn sub_assign(&mut self, rhs: StackPointer) {
        *self = *self - rhs;
    }
}

impl std::ops::Sub<u64> for StackPointer {
    type Output = StackPointer;

    fn sub(self, mut x: u64) -> Self::Output {
        x %= self.stack_size;

        if self.pointer >= x {
            StackPointer::new(self.pointer - x, self.stack_size)
        } else {
            StackPointer::new(self.stack_size - (x - self.pointer), self.stack_size)
        }
    }
}

impl std::ops::SubAssign<u64> for StackPointer {
    fn sub_assign(&mut self, rhs: u64) {
        *self = *self - rhs;
    }
}

impl std::cmp::PartialEq<u64> for StackPointer {
    fn eq(&self, other: &u64) -> bool {
        self.pointer.eq(other)
    }
}

impl std::cmp::PartialOrd<u64> for StackPointer {
    fn partial_cmp(&self, other: &u64) -> Option<std::cmp::Ordering> {
        Some(self.pointer.cmp(other))
    }
}