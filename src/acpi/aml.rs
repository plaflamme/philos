use acpi::{AcpiHandler, AmlTable};
use alloc::boxed::Box;
use aml::{AmlContext, AmlError, DebugVerbosity};

struct Handler;

impl aml::Handler for Handler {
    fn read_u8(&self, address: usize) -> u8 {
        crate::serial_println!("read_u8@{}", address);
        0
    }

    fn read_u16(&self, address: usize) -> u16 {
        crate::serial_println!("read_u16@{}", address);
        0
    }

    fn read_u32(&self, address: usize) -> u32 {
        crate::serial_println!("read_u32@{}", address);
        0
    }

    fn read_u64(&self, address: usize) -> u64 {
        crate::serial_println!("read_u64@{}", address);
        0
    }

    fn write_u8(&mut self, address: usize, value: u8) {
        crate::serial_println!("write_u8@{}={}", address, value);
    }

    fn write_u16(&mut self, address: usize, value: u16) {
        crate::serial_println!("write_u16@{}={}", address, value);
    }

    fn write_u32(&mut self, address: usize, value: u32) {
        crate::serial_println!("write_u32@{}={}", address, value);
    }

    fn write_u64(&mut self, address: usize, value: u64) {
        crate::serial_println!("write_u64@{}={}", address, value);
    }

    fn read_io_u8(&self, port: u16) -> u8 {
        crate::serial_println!("read_io_u8@{}", port);
        0
    }

    fn read_io_u16(&self, port: u16) -> u16 {
        crate::serial_println!("read_io_u16@{}", port);
        0
    }

    fn read_io_u32(&self, port: u16) -> u32 {
        crate::serial_println!("read_io_u32@{}", port);
        0
    }

    fn write_io_u8(&self, port: u16, value: u8) {
        crate::serial_println!("write_io_u8@{}={}", port, value);
    }

    fn write_io_u16(&self, port: u16, value: u16) {
        crate::serial_println!("write_io_u16@{}={}", port, value);
    }

    fn write_io_u32(&self, port: u16, value: u32) {
        crate::serial_println!("write_io_u32@{}={}", port, value);
    }

    fn read_pci_u8(&self, segment: u16, bus: u8, device: u8, function: u8, offset: u16) -> u8 {
        crate::serial_println!(
            "read_pci_u8@{},{},{},{},{}",
            segment,
            bus,
            device,
            function,
            offset
        );
        0
    }

    fn read_pci_u16(&self, segment: u16, bus: u8, device: u8, function: u8, offset: u16) -> u16 {
        crate::serial_println!(
            "read_pci_u16@{},{},{},{},{}",
            segment,
            bus,
            device,
            function,
            offset
        );
        0
    }

    fn read_pci_u32(&self, segment: u16, bus: u8, device: u8, function: u8, offset: u16) -> u32 {
        crate::serial_println!(
            "read_pci_u32@{},{},{},{},{}",
            segment,
            bus,
            device,
            function,
            offset
        );
        0
    }

    fn write_pci_u8(
        &self,
        segment: u16,
        bus: u8,
        device: u8,
        function: u8,
        offset: u16,
        value: u8,
    ) {
        crate::serial_println!(
            "write_pci_u8@{},{},{},{},{}={}",
            segment,
            bus,
            device,
            function,
            offset,
            value
        );
    }

    fn write_pci_u16(
        &self,
        segment: u16,
        bus: u8,
        device: u8,
        function: u8,
        offset: u16,
        value: u16,
    ) {
        crate::serial_println!(
            "write_pci_u16@{},{},{},{},{}={}",
            segment,
            bus,
            device,
            function,
            offset,
            value
        );
    }

    fn write_pci_u32(
        &self,
        segment: u16,
        bus: u8,
        device: u8,
        function: u8,
        offset: u16,
        value: u32,
    ) {
        crate::serial_println!(
            "write_pci_u32@{},{},{},{},{}={}",
            segment,
            bus,
            device,
            function,
            offset,
            value
        );
    }
}

pub(super) fn parse_table(tbl: &AmlTable) -> Result<AmlContext, AmlError> {
    let mut ctx = AmlContext::new(Box::new(Handler {}), false, DebugVerbosity::All);
    let acpi_handler = crate::acpi::Handler {};
    let mapping = unsafe { acpi_handler.map_physical_region(tbl.address, tbl.length as usize) };

    let ptr: *mut u8 = mapping.virtual_start.as_ptr();
    let slice = unsafe { core::slice::from_raw_parts(ptr, tbl.length as usize) };
    ctx.parse_table(slice)?;
    Ok(ctx)
}
