use acpi::sdt::SdtHeader;
use acpi::AcpiTable;

// Unfortunately, the real acpi::fadt is not public

#[repr(C, packed)]
pub struct Fadt {
    header: SdtHeader,

    firmware_ctrl: u32,
    dsdt_address: u32,

    // used in acpi 1.0; compatibility only, should be zero
    _reserved: u8,

    preferred_pm_profile: u8,
    sci_interrupt: u16,
    pub smi_cmd_port: u32,
    pub acpi_enable: u8,
    acpi_disable: u8,
    s4bios_req: u8,
    pstate_control: u8,
    pm1a_event_block: u32,
    pm1b_event_block: u32,
    pub pm1a_control_block: u32,
    pub pm1b_control_block: u32,
    pm2_control_block: u32,
    pm_timer_block: u32,
    gpe0_block: u32,
    gpe1_block: u32,
    pm1_event_length: u8,
    pm1_control_length: u8,
    pm2_control_length: u8,
    pm_timer_length: u8,
    gpe0_block_length: u8,
    gpe1_block_length: u8,
    gpe1_base: u8,
    c_state_control: u8,
    worst_c2_latency: u16,
    worst_c3_latency: u16,
    flush_size: u16,
    flush_stride: u16,
    duty_offset: u8,
    duty_width: u8,
    day_alarm: u8,
    month_alarm: u8,
    century: u8,
    iapc_boot_arch: u16,
    _reserved2: u8, // must be 0
    pub flags: u32,
    //    reset_reg: RawGenericAddress,
    //    reset_value: u8,
    //    arm_boot_arch: u16,
    //    fadt_minor_version: u8,
}

impl AcpiTable for Fadt {
    fn header(&self) -> &SdtHeader {
        &self.header
    }
}
