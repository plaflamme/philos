use core::ops::DerefMut;
use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB};
use x86_64::VirtAddr;

pub mod bump;
pub mod fixed;
pub mod free_list;

#[global_allocator]
static ALLOCATOR: Locked<fixed::FixedAllocator> = Locked::new(fixed::FixedAllocator::new());

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

pub fn init() -> Result<(), MapToError<Size4KiB>> {
    let mut mapper = crate::memory::MAPPER.get().unwrap().lock();
    let mut frame_allocator = crate::memory::FRAME_ALLOCATOR.get().unwrap().lock();
    let page_range = {
        let start = VirtAddr::new(HEAP_START as u64);
        let end = start + (HEAP_SIZE - 1) as u64;
        let start_page = Page::containing_address(start);
        let end_page = Page::containing_address(end);
        Page::range_inclusive(start_page, end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        unsafe {
            mapper
                .map_to(
                    page,
                    frame,
                    PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                    frame_allocator.deref_mut(),
                )?
                .flush();
        }
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}

pub struct Locked<A> {
    value: spin::Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(value: A) -> Self {
        Locked {
            value: spin::Mutex::new(value),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.value.lock()
    }
}

pub fn align_up(addr: usize, align: usize) -> usize {
    let r = addr % align;
    if r == 0 {
        addr
    } else {
        addr - r + align
    }
}
