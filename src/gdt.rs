use lazy_static::lazy_static;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

// https://os.phil-opp.com/double-fault-exceptions/#switching-stacks
lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };
}

struct Selectors {
    kernel_code_segment: SegmentSelector,
    tss_segment: SegmentSelector,
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let kernel_code_segment = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_segment = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (
            gdt,
            Selectors {
                kernel_code_segment,
                tss_segment,
            },
        )
    };
}

pub fn init_gdt() {
    GDT.0.load();

    // https://os.phil-opp.com/double-fault-exceptions/#the-final-steps
    unsafe {
        x86_64::instructions::segmentation::set_cs(GDT.1.kernel_code_segment);
        x86_64::instructions::tables::load_tss(GDT.1.tss_segment);
    }
}
