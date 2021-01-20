use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{PageTable, OffsetPageTable};
use x86_64::VirtAddr;

pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

unsafe fn active_level4_table(phys_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (level4_frame, _) = Cr3::read();

    let phys_addr = level4_frame.start_address();
    let virt = phys_memory_offset + phys_addr.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}
