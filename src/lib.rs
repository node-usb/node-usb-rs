mod webusb_device;

use webusb_device::USBDevice;
use napi_derive::napi;
use nusb::{descriptors::language_id::US_ENGLISH, MaybeFuture};
use std::time::Duration;

/*
  TODO
  - implement getDeviceList()
  - implement findByIds(vid, pid)
  - implement findBySerialNumber(serialNumber)
  - implement hotplug events
  - test!
*/


#[napi]
pub async fn list() -> Vec<USBDevice> {
    nusb::list_devices().wait().unwrap().map(|dev| USBDevice::new(dev)).collect()
}

#[napi]
pub async fn byidasync(vid: u16, pid: u16) -> USBDevice {
    let device = nusb::list_devices()
        .wait()
        .unwrap()
        .find(|dev| dev.vendor_id() == vid && dev.product_id() == pid)
        .expect("device not connected");

    USBDevice::new(device)
}

#[napi]
pub async fn serialbyidasync(vid: u16, pid: u16) -> String {
    let device = nusb::list_devices()
        .wait()
        .unwrap()
        .find(|dev| dev.vendor_id() == vid && dev.product_id() == pid)
        .expect("device not connected");

    let dev = match device.open().wait() {
        Ok(dev) => dev,
        Err(e) => {
            return format!("Failed to open device: {}", e);
        }
    };

    let timeout = Duration::from_millis(100);

    let dev_descriptor = dev.device_descriptor();

    let languages: Vec<u16> = dev.get_string_descriptor_supported_languages(timeout).wait().map(|i| i.collect()).unwrap_or_default();

    let language = languages.first().copied().unwrap_or(US_ENGLISH);

    if let Some(i_serial) = dev_descriptor.serial_number_string_index() {
        let s = dev.get_string_descriptor(i_serial, language, timeout).wait().unwrap();

        return format!("{s:?}");
    }

    return "  No Serial Number".to_string();
}

// watch_devices -> HotplugWatch
