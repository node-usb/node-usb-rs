#![allow(non_snake_case)]

mod webusb_device;

use futures_lite::stream;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;
use nusb::{hotplug::HotplugEvent, MaybeFuture};
use webusb_device::{Handle, UsbDevice};
/*
  TODO
  - tidy error returns
  - fix endpoint read/write
  - test!
*/

#[napi]
pub struct Emitter {
    attachCallback: Option<ThreadsafeFunction<UsbDevice, (), UsbDevice, napi::Status, false>>,
    detachCallback: Option<ThreadsafeFunction<Handle, (), Handle, napi::Status, false>>,
}

#[napi]
impl Emitter {
    #[napi(constructor)]
    pub fn new() -> Self {
        let instance = Self {
            attachCallback: None,
            detachCallback: None,
        };

        let watch = nusb::watch_devices().unwrap();
        for event in stream::block_on(watch) {
            match event {
                HotplugEvent::Connected(info) => {
                    if let Some(callback) = &instance.attachCallback {
                        let device = UsbDevice::new(info);
                        callback.call(device, ThreadsafeFunctionCallMode::NonBlocking);
                    }
                }
                HotplugEvent::Disconnected(id) => {
                    if let Some(callback) = &instance.detachCallback {
                        let handle = Handle::from_nusb(id);
                        callback.call(handle, ThreadsafeFunctionCallMode::NonBlocking);
                    }
                }
            }
        }

        instance
    }

    #[napi]
    pub fn onAttach(&mut self, callback: ThreadsafeFunction<UsbDevice, (), UsbDevice, napi::Status, false>) {
        self.attachCallback = Some(callback);
    }

    #[napi]
    pub fn onDetach(&mut self, callback: ThreadsafeFunction<Handle, (), Handle, napi::Status, false>) {
        self.detachCallback = Some(callback);
    }
}

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
