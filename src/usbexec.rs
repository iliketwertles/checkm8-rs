use std::{thread::sleep, time::{Duration, SystemTime, UNIX_EPOCH}, ptr::NonNull};
use libusb1_sys::{libusb_fill_control_setup, libusb_fill_control_transfer, libusb_alloc_transfer, libusb_open_device_with_vid_pid, libusb_transfer, libusb_submit_transfer, libusb_cancel_transfer, libusb_control_transfer, libusb_init, libusb_get_device, libusb_get_device_descriptor, libusb_device_descriptor, libusb_get_string_descriptor_ascii, libusb_device_handle, libusb_get_device_list, libusb_open};
use rusb::{DeviceHandle, Context};

extern "system" fn _dummy(_: *mut libusb_transfer) {}

pub fn is_pwn_dfu(dev: *mut libusb_device_handle) -> bool {
    let mut serialnumber: u8 = 0;
    let mut desc: libusb_device_descriptor = unsafe { std::mem::zeroed() };
    unsafe {
        let device = libusb_get_device(dev);
        let _r = libusb_get_device_descriptor(device, &mut desc);
        let _r = libusb_get_string_descriptor_ascii(dev, desc.iSerialNumber, &mut serialnumber, 1);
        println!("SerialNumber: {}", serialnumber);
        if serialnumber.to_string().contains("PWND") {
            return true
        } else {
            return false
        }
    }
}

pub fn aquire_device(vid: u16, pid: u16) -> *mut libusb_device_handle {
    println!("aquire");
    unsafe {
        let desc: *mut libusb_device_descriptor = std::mem::zeroed();
        let mut r = -1;
        let handle: *mut *mut libusb_device_handle = std::mem::zeroed();
        //let list: *mut *const *mut libusb_device = std::mem::zeroed();
        let mut list = vec![];
        let p = list.as_mut_ptr();
        list.set_len(libusb_get_device_list(std::ptr::null_mut(), p).try_into().unwrap());
        dbg!(&list);
        libusb_init(std::ptr::null_mut());
        println!("init'd");

        //let list_len = libusb_get_device_list(std::ptr::null_mut(), transmute(list[0]));
        println!("dev list");
        let list = list;

        for device in list {
            libusb_get_device_descriptor(device.cast(), desc);
            println!("desc");
            if (*desc).idVendor == vid {
                if (*desc).idProduct == pid {
                    libusb_open(device.cast(), handle);
                    println!("opened");
                    break;
                }
            }
        }

        //libusb_get_device_list(std::ptr::null_mut(), list as *mut *const *mut libusb_device);

        //let dev = libusb_open_device_with_vid_pid(std::ptr::null_mut(), vid, pid);
        //for device in DeviceList::new().unwrap().iter() {
        //    let device_desc = match device.device_descriptor() {
        //        Ok(d) => d,
        //        Err(_) => continue,
        //    };
        //    if device_desc.vendor_id() == 0x5ac {
        //        println!("vid match");
        //    }
        //    if device_desc.product_id() == 0x1227 {
        //        println!("pid match");
        //    }
            
        //}
        //let dev = libusb_open_device_with_vid_pid(std::ptr::null_mut(), vid, pid);
        sleep(Duration::from_secs(1));

        let device1 = libusb_get_device(handle.cast());

        //r = libusb_claim_interface(dev, 0);
        //if r < 0 {
        //    panic!("libusb_claim_interface error")
        //}

        r = libusb_get_device_descriptor(device1, desc);
        if r < 0 {
            panic!("libusb_get_device_descriptor error");
        }

        let mut serialnumber: u8 = 0;
        r = libusb_get_string_descriptor_ascii(handle.cast(), (*desc).iSerialNumber, &mut serialnumber, 1);
        if r < 0 {
            panic!("libusb_get_string_descriptor_ascii error");
        }
        return handle.cast()
    }
}

pub fn usb_req_stall(vid: u16, pid: u16) {
    unsafe {
        let han = libusb_open_device_with_vid_pid(std::ptr::null_mut(), vid , pid);
        libusb_control_transfer(han, 0x2, 3, 0x0, 0x80, std::ptr::null_mut(), 0, 10);
    }
}

pub fn usb_req_leak(vid: u16, pid: u16) {
    unsafe {
        let han = libusb_open_device_with_vid_pid(std::ptr::null_mut(), vid , pid);
        let mut buf: [u8; 0x40] = [0; 0x40];
        libusb_control_transfer(han, 0x80 , 6, 0x304, 0x40A, buf.as_mut_ptr(), 0x40, 1);
    }
}

pub fn usb_no_leak(vid: u16, pid: u16) {
    unsafe {
        let han = libusb_open_device_with_vid_pid(std::ptr::null_mut(), vid , pid);
        let mut buffer_size: [u8; 0xc1] = [0; 0xc1];
        libusb_control_transfer(han, 0x80, 6, 0x304, 0x40A, buffer_size.as_mut_ptr(), 0xC1, 1);
    }
}

pub fn usb_stall(vid: u16, pid: u16) {
    let buffer_size: usize = 0xC0;
    let buf: Vec<u8> = vec![b'A'; buffer_size];
    let _ret = async_ctrl_transfer( 0x80, 6, 0x304, 0x40A, &buf, 0xC0, 0.00001, vid, pid);
}

fn get_nanos() -> u64 {
    let start = SystemTime::now();
    let since_epoch = start.duration_since(UNIX_EPOCH).unwrap();
    since_epoch.as_secs() * 1_000_000_000 + since_epoch.subsec_nanos() as u64
}

pub fn async_ctrl_transfer(bm_request_type: u8, b_request: u8, w_value: u16, w_index: u16, data: &[u8], w_length: u16, timeout: f32, vid: u16, pid: u16) -> i32 {
    let start = get_nanos();
    let mut buffer = vec![0; w_length as usize + 8];
    buffer[8..].copy_from_slice(data);
    unsafe {
        let han = libusb_open_device_with_vid_pid(std::ptr::null_mut(), vid , pid);
        libusb_fill_control_setup(buffer.as_mut_ptr(), bm_request_type, b_request, w_value, w_index, w_length);
        let transfer = libusb_alloc_transfer(0);
        libusb_fill_control_transfer(transfer, han, buffer.as_mut_ptr(), _dummy, std::ptr::null_mut(), 0);
        let ret = libusb_submit_transfer(transfer);
        if ret != 0 {
            return -1
        }
        while get_nanos() as f32 - (start as f32) < (timeout * (10.0 * 6.0)) {}
        let ret = libusb_cancel_transfer(transfer);
        if ret != 0 {
            return -1
        } else {
            return 0
        }
    }
}

pub fn reconnect_device(bad_handle: NonNull<libusb_device_handle>, vid: u16, pid: u16, timer: Duration) -> *mut libusb_device_handle {
    println!("Reconnecting device...");
    unsafe {
        let normal_handle = Context::new().unwrap();
        let mut normal_handle = DeviceHandle::<rusb::Context>::from_libusb(normal_handle, bad_handle);
        normal_handle.release_interface(0).expect("Unable to release device.");
    }
    sleep(timer);
    return aquire_device(vid, pid)
}