use x86_64::registers::control::Cr3;
use x86_64::structures::paging::PageTable;
use x86_64::structures::paging::page_table::FrameError;
use x86_64::{VirtAddr, PhysAddr};

pub unsafe fn active_level4_table(phys_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (level4_frame, _) = Cr3::read();

    let phys_addr = level4_frame.start_address();
    let virt = phys_memory_offset + phys_addr.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}

pub unsafe fn translate_addr(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr>
{
    translate_addr_inner(addr, physical_memory_offset)
}

fn translate_addr_inner(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    let offsets = [
        addr.p4_index(), addr.p3_index(), addr.p2_index(), addr.p1_index()
    ];

    let (mut frame, _) = Cr3::read();

    for &offset in &offsets {
        let phys_addr = frame.start_address();
        let virt_addr = physical_memory_offset + phys_addr.as_u64();
        let page_table_ptr: *const PageTable = virt_addr.as_ptr();
        let page_table: &PageTable = unsafe { & *page_table_ptr };
        let page_table_entry = &page_table[offset];

        frame = match page_table_entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => unimplemented!()
        }
    }

    Some(frame.start_address() + u64::from(addr.page_offset()))
}
