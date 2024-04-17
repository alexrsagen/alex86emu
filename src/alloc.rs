use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

use bytesize::ByteSize;

pub type SystemTrackingAllocator = TrackingAllocator<System>;

pub struct TrackingAllocator<A: GlobalAlloc> {
    allocator: A,
    size: AtomicUsize,
}

unsafe impl<A: GlobalAlloc> GlobalAlloc for TrackingAllocator<A> {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        self.size.fetch_add(l.size(), Ordering::SeqCst);
        self.allocator.alloc(l)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, l: Layout) {
        self.allocator.dealloc(ptr, l);
        self.size.fetch_sub(l.size(), Ordering::SeqCst);
    }
}

impl<A: GlobalAlloc> TrackingAllocator<A> {
    pub const fn new(allocator: A) -> Self {
        let size = AtomicUsize::new(0);
        Self { allocator, size }
    }
    #[allow(unused)]
    pub fn reset(&self) {
        self.size.store(0, Ordering::SeqCst);
    }
    pub fn get(&self) -> usize {
        self.size.load(Ordering::SeqCst)
    }
    pub fn get_bytesize(&self) -> ByteSize {
        ByteSize::b(self.get() as u64)
    }
}

impl SystemTrackingAllocator {
    pub const fn new_system() -> Self {
        Self::new(System)
    }
}
