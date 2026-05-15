#![no_main]
#![no_std]

mod kernel_args;
mod identity_acpi_handler;

use crate::kernel_args::{KernelArgs, OSMemEntry};
use crate::identity_acpi_handler::IdentityAcpiHandler;
use acpi::AcpiTables;
use acpi::mcfg::PciConfigRegions;
use uefi::mem::memory_map::MemoryMap;
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

    //walk through all 66356 possible segment groups
    for segment_group in 0u16..=65535u16{
       if let Some(addr) = pcie_cfg.physical_address(segment_group, 0, 0, 0) {
            kargs.set_pcie(addr as *mut core::ffi::c_void);
            break;
        }
    }
 
    info!("ACPI Revision: {}", acpi_tables.revision);
    
    let (mm_ptr, total_entries) = get_mm();
    kargs.set_memmap(mm_ptr, total_entries);
    info!("Memory adquired");
    info!("Kernel Arguments: {:?}", kargs);

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

//get memory map. get the memory map and then allocate the pool. Once allocated read it and parse it.
fn get_mm() -> (*mut OSMemEntry, usize) {
    let mm = boot::memory_map(uefi::mem::memory_map::MemoryType::LOADER_DATA).unwrap();
    let mm_size = mm.len() * core::mem::size_of::<OSMemEntry>(); 
    let mm_ptr = boot::allocate_pool(uefi::mem::memory_map::MemoryType::LOADER_DATA, mm_size)
        .unwrap()
        .as_ptr() as *mut OSMemEntry;
    
    let mm_entries = unsafe {
        core::slice::from_raw_parts_mut::<OSMemEntry>(mm_ptr, mm.len())
    };

    let mut total_entries = 0;
    for (idx, entry) in mm.entries().enumerate() {
        mm_entries[idx] = entry.into();
        total_entries += 1;
    };
    
    (mm_ptr, total_entries)
}

impl From<&boot::MemoryDescriptor> for OSMemEntry{
    fn from(mdesc: &boot::MemoryDescriptor) -> OSMemEntry {
        OSMemEntry{
            ty: mdesc.ty,
            base: mdesc.phys_start as usize,
            pages: mdesc.page_count as usize,
            att: mdesc.att,
        }
    }
}
