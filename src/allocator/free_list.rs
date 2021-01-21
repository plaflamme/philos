use crate::allocator::Locked;
use core::alloc::{GlobalAlloc, Layout};
use core::mem;
use core::ptr::null_mut;

struct Node {
    size: usize,
    next: Option<&'static mut Node>,
}

impl Node {
    const fn new(size: usize) -> Self {
        Node { size, next: None }
    }

    fn start_addr(&self) -> usize {
        self as *const Self as usize
    }

    fn end_addr(&self) -> usize {
        self.start_addr() + self.size
    }
}

pub struct FreeListAllocator {
    head: Node,
}

impl FreeListAllocator {
    pub const fn new() -> Self {
        FreeListAllocator { head: Node::new(0) }
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.add_free_region(heap_start, heap_size);
    }

    unsafe fn add_free_region(&mut self, addr: usize, size: usize) {
        // ensure that the freed region is capable of holding Node
        assert_eq!(super::align_up(addr, mem::align_of::<Node>()), addr);
        let mut node = Node::new(size);
        node.next = self.head.next.take();
        let node_ptr = addr as *mut Node;
        node_ptr.write(node);
        self.head.next = Some(&mut *node_ptr);
    }

    fn find_region(&mut self, size: usize, align: usize) -> Option<(&'static mut Node, usize)> {
        let mut current = &mut self.head;
        while let Some(ref mut node) = current.next {
            if let Ok(alloc_start) = Self::align_from_region(node, size, align) {
                // unlink
                let next = node.next.take();
                let found = current.next.take().unwrap();
                current.next = next;
                return Some((found, alloc_start));
            } else {
                current = current.next.as_mut().unwrap();
            }
        }
        None
    }

    fn align_from_region(region: &Node, size: usize, align: usize) -> Result<usize, ()> {
        let alloc_start = super::align_up(region.start_addr(), align);
        let alloc_end = alloc_start.checked_add(size).ok_or(())?;

        if alloc_end > region.end_addr() {
            return Err(());
        }

        let excess = region.end_addr() - alloc_end;
        // if not perfect fit, we must have enough room to store a Node
        if excess > 0 && excess < mem::size_of::<Node>() {
            return Err(());
        }

        Ok(alloc_start)
    }

    fn size_align(layout: Layout) -> (usize, usize) {
        let layout = layout
            .align_to(mem::align_of::<Node>())
            .expect("cannot align")
            .pad_to_align();

        let size = layout.size().max(mem::size_of::<Node>());
        (size, layout.align())
    }
}

unsafe impl GlobalAlloc for Locked<FreeListAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let (size, align) = FreeListAllocator::size_align(layout);
        let mut allocator = self.lock();

        if let Some((region, addr)) = allocator.find_region(size, align) {
            let alloc_end = addr.checked_add(size).expect("overflow");
            let excess = region.end_addr() - addr;
            if excess > 0 {
                allocator.add_free_region(alloc_end, excess);
            }
            addr as *mut u8
        } else {
            null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let (size, _) = FreeListAllocator::size_align(layout);
        self.lock().add_free_region(ptr as usize, size);
    }
}
