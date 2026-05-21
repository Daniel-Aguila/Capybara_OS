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
use uefi::proto::console::gop::{GraphicsOutput, Mode, PixelFormat};
use log::info;
use uefi::{ResultExt, boot};
use uefi::{entry, Status};

const SEGMENT_GROUPS: u16 = 65535;
const MAX_WIDTH: usize = 1920;
const MAX_HEIGHT: usize = 1080;

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    let handle = uefi::boot::image_handle();
    //initialize frame buffer

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

    for segment_group in 0u16..= SEGMENT_GROUPS{
       if let Some(addr) = pcie_cfg.physical_address(segment_group, 0, 0, 0) {
            kargs.set_pcie(addr as *mut core::ffi::c_void);
            break;
        }
    }
 
    info!("ACPI Revision: {}", acpi_tables.revision);
    
    //get memory map and load it
    let (mm_ptr, total_entries) = get_mm();
    kargs.set_memmap(mm_ptr, total_entries);
    info!("Memory adquired");
    info!("Kernel Arguments: {:?}", kargs);

    uefi::system::with_stdin(|input| {
        let _ = read_keyboard_events(input);
    });
    
    //initialize init_framebuffer
    init_framebuffer();

    //No error exit
    Status::SUCCESS
}

//using https://blog.malware.re/2023/11/12/rust-os-part3/index.html as reference
fn init_framebuffer(){
    let mut gfx = boot::get_handle_for_protocol::<GraphicsOutput>()
        .and_then(|op|
            boot::open_protocol_exclusive::<GraphicsOutput>(op)
        )
        .unwrap();
    let mut cur_mode: Option<Mode> = None; 
    let mut cur_width: usize = 0;
    let mut cur_height: usize = 0;

    for (idx, mode) in gfx.modes().enumerate(){
        let mode_info = mode.info();
        let mode_pxl_fmt = mode.info().pixel_format();

        if (mode_pxl_fmt != PixelFormat::Rgb && mode_pxl_fmt != PixelFormat::Bgr){
            continue;
        }

        let (temp_width, temp_height) = mode_info.resolution();
        if (temp_width > cur_width && temp_width <= MAX_WIDTH || temp_height > cur_height && temp_height <= MAX_HEIGHT){
            cur_mode = Some(mode);
            cur_width = temp_width;
            cur_height = temp_height;
        }
    }

    let _ = gfx.set_mode(&cur_mode.unwrap());
}

fn read_keyboard_events(input: &mut Input) -> uefi::Result<()>{
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

impl From<&uefi::mem::memory_map::MemoryDescriptor> for OSMemEntry{
    fn from(mdesc: &uefi::mem::memory_map::MemoryDescriptor) -> OSMemEntry {
        OSMemEntry{
            ty: mdesc.ty,
            base: mdesc.phys_start as usize,
            pages: mdesc.page_count as usize,
            att: mdesc.att,
        }
    }
}
