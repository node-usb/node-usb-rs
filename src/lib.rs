#![allow(non_snake_case)]

mod webusb_device;

use napi_derive::napi;
use nusb::MaybeFuture;
use webusb_device::UsbDevice;

/*
  TODO
  - tidy error returns
  - implement hotplug events: watch_devices -> HotplugWatch
  - fix endpoint read/write
  - test!
*/

#[napi(js_name = "nativeGetDeviceList")]
pub async fn getDeviceList() -> Vec<UsbDevice> {
    nusb::list_devices().wait().unwrap().map(|dev| UsbDevice::new(dev)).collect()
}

#[napi(js_name = "nativeFindByIds")]
pub async fn findByIds(vendorId: u16, productId: u16) -> UsbDevice {
    let device = nusb::list_devices()
        .wait()
        .unwrap()
        .find(|dev| dev.vendor_id() == vendorId && dev.product_id() == productId)
        .expect("device not found");

    UsbDevice::new(device)
}

#[napi(js_name = "nativeFindBySerialNumber")]
pub async fn findBySerialNumber(serialNumber: String) -> UsbDevice {
    let device = nusb::list_devices()
        .wait()
        .unwrap()
        .find(|dev| dev.serial_number() == Some(serialNumber.as_str()))
        .expect("device not found");

    UsbDevice::new(device)
}
