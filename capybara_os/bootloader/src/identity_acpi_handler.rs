use acpi::{AcpiHandler, PhysicalMapping};

#[derive(Clone)]
pub struct IdentityAcpiHandler;

impl AcpiHandler for IdentityAcpiHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize
    ) -> PhysicalMapping<Self, T>{
        //Ring 0 allows us to return the data requested back to the caller
        //we just return the physical since it is a match 1 to 1 with virtual
        //using blog.malware.re/2023/09/01/rust-os-part2/index.html
        unsafe{
            PhysicalMapping::new(
                physical_address,
                core::ptr::NonNull::<T>::new_unchecked(physical_address as *mut T),
                size,
                size,
                Self,
            )
        }
    }
    
    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self,T>){ }
}
