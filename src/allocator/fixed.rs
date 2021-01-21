use crate::allocator::Locked;
use core::alloc::{GlobalAlloc, Layout};
use core::mem;
use core::ptr::{null_mut, NonNull};

const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];

fn block_list_index(layout: &Layout) -> Option<usize> {
    let size = layout.size().max(layout.align());
    BLOCK_SIZES.iter().position(|&b| b > size)
}

struct Node {
    next: Option<&'static mut Node>,
}

impl Node {
    fn new() -> Self {
        Node { next: None }
    }
}

pub struct FixedAllocator {
    heads: [Option<&'static mut Node>; BLOCK_SIZES.len()],
    fallback: linked_list_allocator::Heap,
}

impl FixedAllocator {
    pub const fn new() -> Self {
        FixedAllocator {
            heads: [None; BLOCK_SIZES.len()],
            fallback: linked_list_allocator::Heap::empty(),
        }
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.fallback.init(heap_start, heap_size);
    }

    fn fallback_alloc(&mut self, layout: Layout) -> *mut u8 {
        match self.fallback.allocate_first_fit(layout) {
            Ok(ptr) => ptr.as_ptr(),
            Err(_) => null_mut(),
        }
    }
}

unsafe impl GlobalAlloc for Locked<FixedAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut alloc = self.lock();
        match block_list_index(&layout) {
            Some(idx) => match alloc.heads[idx].take() {
                Some(blk) => {
                    alloc.heads[idx] = blk.next.take();
                    blk as *mut Node as *mut u8
                }
                None => {
                    let block_size = BLOCK_SIZES[idx];
                    let block_align = block_size;
                    let layout = Layout::from_size_align(block_size, block_align).unwrap();
                    alloc.fallback_alloc(layout)
                }
            },
            None => alloc.fallback_alloc(layout),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut alloc = self.lock();
        match block_list_index(&layout) {
            Some(idx) => {
                let mut node = Node::new();
                node.next = alloc.heads[idx].take();

                assert!(mem::size_of::<Node>() <= BLOCK_SIZES[idx]);
                assert!(mem::align_of::<Node>() <= BLOCK_SIZES[idx]);

                let node_ptr = ptr as *mut Node;
                node_ptr.write(node);
                alloc.heads[idx] = Some(&mut *node_ptr);
            }
            None => alloc
                .fallback
                .deallocate(NonNull::new(ptr).unwrap(), layout),
        }
    }
}
