use core::ffi::c_void;
use uefi::table::cfg::{ConfigTableEntry};

//Memory Mapping structure
pub struct OSMemEntry{
    pub ty: uefi::mem::memory_map::MemoryType,
    pub base: usize,
    pub pages: usize,
    pub att: uefi::mem::memory_map::MemoryAttribute
}

//acpi_ptr: acpi physical address, smbios_ptr: smbios physical address, and their versions
#[derive(Copy, Clone, Debug)]
pub struct KernelArgs {
    acpi_ptr: *const c_void,
    smbios_ptr: *const c_void, acpi_ver: u8, smbios_ver: u8,
    pcie_ptr: *mut c_void,
    memmap_entries: usize,
    memmap_ptr: *mut OSMemEntry,
}

//Default values
impl Default for KernelArgs {
    fn default() -> Self {
        Self {
            acpi_ptr: 0 as *const c_void,
            smbios_ptr: 0 as *const c_void,
            acpi_ver: 0,
            smbios_ver: 0,
            pcie_ptr: 0 as *mut c_void,
            memmap_entries: 0,
            memmap_ptr: core::ptr::null_mut()
        }
    }
}


//populate the KernelArgs structure from the CFG table
impl KernelArgs {
    pub fn populate_from_cfg_table(&mut self, cfg_tables: &[ConfigTableEntry]){
        for cfg in cfg_tables {
            match cfg.guid {
                ConfigTableEntry::ACPI2_GUID => {
                    if self.acpi_ver < 2 {
                        self.acpi_ver = 2;
                        self.acpi_ptr = cfg.address;
                    }
                }
                ConfigTableEntry::ACPI_GUID => {
                    if self.acpi_ver < 1 {
                        self.acpi_ver = 1;
                        self.acpi_ptr = cfg.address;
                    }
                }
                ConfigTableEntry::SMBIOS3_GUID => {
                    if self.smbios_ver < 3 {
                        self.smbios_ver = 3;
                        self.smbios_ptr = cfg.address;
                    }
                }
                ConfigTableEntry::SMBIOS_GUID => {
                    if self.smbios_ver < 1 {
                        self.smbios_ver = 1;
                        self.smbios_ptr = cfg.address;
                    }
                }
                _ => {},
            }
        }
    }

    //return acpi physical address and version
    pub fn get_acpi(&self) -> (*const c_void, u8) {
        (self.acpi_ptr, self.acpi_ver) 
    }
    //return smbios physical address and version
    pub fn get_smbios(&self) -> (*const c_void, u8) {
        (self.smbios_ptr, self.smbios_ver)
    }
    //set and get PCI Express ECAM pointer
    pub fn set_pcie(&mut self,ptr: *mut c_void){
        self.pcie_ptr = ptr
    }
    pub fn get_pcie(&self) -> *mut c_void{
        self.pcie_ptr
    }
    //set memory mapping pointer and slice size
    pub fn set_memmap(&mut self, ptr: *mut OSMemEntry, entries: usize){
        self.memmap_ptr = ptr;
        self.memmap_entries = entries;
    }
    //get memory mapping pointer
    pub fn get_memmap(&self) -> *mut OSMemEntry{
        self.memmap_ptr
    }
    //get memory entries number
    pub fn get_memmap_entries(&self) -> usize {
        self.memmap_entries
    }
}
