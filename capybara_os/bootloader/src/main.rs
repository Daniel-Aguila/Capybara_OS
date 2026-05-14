#![no_main]
#![no_std]

mod kernel_args;
mod identity_acpi_handler;

use crate::kernel_args::KernelArgs;
use crate::identity_acpi_handler::IdentityAcpiHandler;
use acpi::AcpiTables;
use acpi::mcfg::PciConfigRegions;
use uefi::proto::console::text::{Input};
use uefi::prelude::*;
use log::info;

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    let handle = uefi::boot::image_handle();
    let revision = uefi::system::uefi_revision();

    info!("Image Handle: {:#018x}", handle.as_ptr() as usize);
    info!("UEFI Provisions: {}.{}",
        revision.major(),
        revision.minor()
    );
    
    //kernel arguments
    let mut kargs = KernelArgs::default();
    info!("Empty karg: {:?}", kargs);
    
    let _ = uefi::system::with_config_table(|tables| {
        kargs.populate_from_cfg_table(tables);
    });

    //parse ACPI data
    let identity_handler = IdentityAcpiHandler;
    let acpi_tables = unsafe{
        AcpiTables::from_rsdp(identity_handler, kargs.get_acpi().0 as usize) 
    }.unwrap();

    let pcie_cfg = PciConfigRegions::new(&acpi_tables).unwrap();
    let pcie_first_addr = pcie_cfg.physical_address(0,0,0,0).unwrap();

    info!("Populated karg: {:?}", kargs);
    info!("ACPI Revision: {}", acpi_tables.revision);
    info!("PCIe(0, 0, 0, 0): {:#018x}", pcie_first_addr);

    uefi::system::with_stdin(|input| {
        let _ = read_keyboard_events(input);
    });
    

    //No error exit
    Status::SUCCESS
}

fn read_keyboard_events(input: &mut Input) -> uefi::Result{
        info!("Press a key to continue...");
        input.reset(true)?; 

        let mut events = [input.wait_for_key_event().unwrap()];
        boot::wait_for_event(&mut events).discard_errdata()?;
        Ok(())
}
