use acpi::PhysicalMapping;
use core::cmp::max;
use core::mem::size_of;
use core::ops::DerefMut;
use core::ptr::NonNull;
use x86_64::structures::paging::frame::PhysFrameRangeInclusive;
use x86_64::structures::paging::page::PageRangeInclusive;
use x86_64::structures::paging::{
    Mapper, Page, PageTableFlags, PhysFrame, Size4KiB,
};
use x86_64::{PhysAddr, VirtAddr};
use core::sync::atomic::{AtomicU64, Ordering};

static ACPI_START: u64 = 0x_3333_3333_0000;
static NEXT_OFFSET: AtomicU64 = AtomicU64::new(0);

#[derive(Clone)]
pub struct Handler;

impl Handler {
    fn page_range_for_size(size: usize) -> PageRangeInclusive {
        let offset = NEXT_OFFSET.load(Ordering::Relaxed);
        let virt_start = VirtAddr::new(ACPI_START + offset);
        let virt_end = virt_start + (size - 1);
        let range: PageRangeInclusive<Size4KiB> = Page::range_inclusive(
            Page::containing_address(virt_start),
            Page::containing_address(virt_end),
        );
        let virt_size: u64 = range.map(|p| p.size()).sum();
        let new = offset + virt_size;
        match NEXT_OFFSET.compare_exchange(offset, new, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => range,
            Err(_) => Handler::page_range_for_size(size),
        }
    }
}

impl acpi::AcpiHandler for Handler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<Self, T> {
        let actual_size = max(size, size_of::<T>());

        let page_range = Handler::page_range_for_size(actual_size);
        let virt_start = page_range.start.start_address();
        let virt_size: u64 = page_range.map(|p| p.size()).sum();

        let phys_start = PhysAddr::new(physical_address as u64);
        let phys_end = phys_start + (actual_size - 1);
        let phys_range: PhysFrameRangeInclusive<Size4KiB> = PhysFrame::range_inclusive(
            PhysFrame::containing_address(phys_start),
            PhysFrame::containing_address(phys_end),
        );
        let phys_size: u64 = phys_range.map(|p| p.size()).sum();

        let locked = &mut crate::memory::FRAME_ALLOCATOR.get().unwrap().lock();
        for (page, phys_frame) in page_range.into_iter().zip(phys_range) {
            let mut mapper = crate::memory::MAPPER.get().expect("memory module not initialized").lock();
            mapper
                .map_to(
                    page,
                    phys_frame,
                    PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                    locked.deref_mut(),
                )
                .expect("failed to map page")
                .flush();
        }
        // align the virtual address to point at the requested physical_address
        let phys_offset = phys_start - phys_range.start.start_address();
        let virtual_address = virt_start + phys_offset;
        PhysicalMapping {
            physical_start: physical_address,
            virtual_start: NonNull::new_unchecked(virtual_address.as_mut_ptr::<T>()),
            region_length: phys_size as usize,
            mapped_length: virt_size as usize,
            handler: self.clone(),
        }
    }

    fn unmap_physical_region<T>(&self, region: &PhysicalMapping<Self, T>) {
        let mut mapper = crate::memory::MAPPER.get().unwrap().lock();
        let virt_start = VirtAddr::from_ptr(region.virtual_start.as_ptr());
        let page_start: Page<Size4KiB> = Page::containing_address(virt_start);
        let virt_end = page_start.start_address() + (region.mapped_length - 1);
        let range: PageRangeInclusive<Size4KiB> =
            Page::range_inclusive(page_start, Page::containing_address(virt_end));

        for page in range {
            match mapper.unmap(page) {
                Ok(f) => f.1.flush(),
                Err(e) => panic!("{:?}", e),
            }
        }
    }
}

pub unsafe fn init() -> acpi::AcpiTables<Handler> {
    match acpi::AcpiTables::search_for_rsdp_bios(Handler) {
        Ok(tables) => tables,
        Err(err) => panic!("failed to initialize ACPI: {:?}", err),
    }
}
