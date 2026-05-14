use core::format_args;
use uefi::{guid, Guid, table::cfg::ConfigTableEntry};

#[derive(Debug)]
pub struct CfgTableType(uefi::Guid);
pub const UEFI_MEMORY_ATTRIBUTES_TABLE:Guid = guid!("dcfa911d-26eb-469f-a220-38b7dc461220");

impl From<Guid> for CfgTableType {
    fn from(guid: Guid) -> Self{
        Self(guid)
    }
}

//Display trait for the GUID properties

impl core::fmt::Display for CfgTableType {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        match self.0 {
            ConfigTableEntry::ACPI_GUID => f.write_str("ACPI1"),
            ConfigTableEntry::ACPI2_GUID => f.write_str("ACPI2"), 
            ConfigTableEntry::DEBUG_IMAGE_INFO_GUID => f.write_str("Debug Image"),
            ConfigTableEntry::DXE_SERVICES_GUID => f.write_str("DXE Services"),
            ConfigTableEntry::ESRT_GUID => f.write_str("EFI System Resources"),
            ConfigTableEntry::HAND_OFF_BLOCK_LIST_GUID => f.write_str("Hand-off Block list"),
            ConfigTableEntry::LZMA_COMPRESS_GUID => f.write_str("LZMA Compressed filesystem"),
            ConfigTableEntry::MEMORY_STATUS_CODE_RECORD_GUID => f.write_str("Hand-off Status Code"),
            ConfigTableEntry::MEMORY_TYPE_INFORMATION_GUID => f.write_str("Memory Type Information"),
            ConfigTableEntry::PROPERTIES_TABLE_GUID => f.write_str("Properties Table"),
            ConfigTableEntry::SMBIOS3_GUID => f.write_str("SMBIOS3"),
            ConfigTableEntry::SMBIOS_GUID => f.write_str("SMBIOS1"),
            ConfigTableEntry::TIANO_COMPRESS_GUID => f.write_str("Tiano compressed filesystem"),
            UEFI_MEMORY_ATTRIBUTES_TABLE => f.write_str("Memory Attributes"),
            x => f.write_fmt(format_args!("{}", x)),
        }
    }
}
