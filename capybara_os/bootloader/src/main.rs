#![no_main]
#![no_std]

use uefi::{Char16, proto::console::text::{Input, Key, ScanCode}};
use uefi::prelude::*;
use log::info;

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    let handle = uefi::boot::image_handle();

    info!("Image Handle: {:#018x}", handle.as_ptr() as usize);
    uefi::system::with_stdin(|input| {
    let _ = read_keyboard_events(input);
    });
    Status::SUCCESS
}

fn read_keyboard_events(input: &mut Input) -> uefi::Result{
        info!("Press a key to continue...");
        input.reset(true)?;     

        let mut events = [input.wait_for_key_event().unwrap()];
        boot::wait_for_event(&mut events).discard_errdata()?;
        Ok(())
}
