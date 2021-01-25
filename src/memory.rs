use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use bootloader::BootInfo;
use conquer_once::spin::OnceCell;
use spin::Mutex;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};

pub static MAPPER: OnceCell<Mutex<OffsetPageTable<'static>>> = OnceCell::uninit();
pub static FRAME_ALLOCATOR: OnceCell<Mutex<BootInfoFrameAllocator>> = OnceCell::uninit();

pub unsafe fn init(boot_info: &'static BootInfo) {
    let phys_offset = VirtAddr::new(boot_info.physical_memory_offset);
    MAPPER.init_once(|| {
        let level_4_table = active_level4_table(phys_offset);
        Mutex::new(OffsetPageTable::new(level_4_table, phys_offset))
    });
    FRAME_ALLOCATOR.init_once(|| Mutex::new(BootInfoFrameAllocator::new(boot_info)))
}

unsafe fn active_level4_table(phys_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (level4_frame, _) = Cr3::read();

    let phys_addr = level4_frame.start_address();
    let virt = phys_memory_offset + phys_addr.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}

pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    unsafe fn new(boot_info: &'static BootInfo) -> Self {
        BootInfoFrameAllocator {
            memory_map: &boot_info.memory_map,
            next: 0,
        }
    }

    fn unused_frames(&self) -> impl Iterator<Item = PhysFrame> {
        self.memory_map
            .iter()
            .filter(|r| r.region_type == MemoryRegionType::Usable)
            .map(|r| r.range.start_addr()..r.range.end_addr())
            .flat_map(|r| r.step_by(4_096))
            .map(PhysAddr::new)
            .map(|r| PhysFrame::containing_address(r))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.unused_frames().nth(self.next);
        self.next += 1;
        frame
    }
}
