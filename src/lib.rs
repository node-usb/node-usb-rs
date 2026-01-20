#![allow(non_snake_case)]

mod webusb_device;

use napi_derive::napi;
use nusb::MaybeFuture;
use webusb_device::USBDevice;

/*
  TODO
  - tidy error returns
  - implement hotplug events: watch_devices -> HotplugWatch
  - test!
*/

#[napi]
pub async fn getDeviceList() -> Vec<USBDevice> {
    nusb::list_devices().wait().unwrap().map(|dev| USBDevice::new(dev)).collect()
}

#[napi]
pub async fn findByIds(vendorId: u16, productId: u16) -> USBDevice {
    let device = nusb::list_devices()
        .wait()
        .unwrap()
        .find(|dev| dev.vendor_id() == vendorId && dev.product_id() == productId)
        .expect("device not found");

    USBDevice::new(device)
}

#[napi]
pub async fn findBySerialNumber(serialNumber: String) -> USBDevice {
    let device = nusb::list_devices()
        .wait()
        .unwrap()
        .find(|dev| dev.serial_number() == Some(serialNumber.as_str()))
        .expect("device not found");

    USBDevice::new(device)
}
