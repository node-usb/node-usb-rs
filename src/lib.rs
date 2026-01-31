#![allow(non_snake_case)]

mod webusb_device;

use futures_lite::StreamExt;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;
use nusb::{hotplug::HotplugEvent, MaybeFuture};
use webusb_device::{UsbDevice};

#[napi]
pub struct Emitter {
    attachCallback: Option<ThreadsafeFunction<UsbDevice, (), UsbDevice, napi::Status, false>>,
    detachCallback: Option<ThreadsafeFunction<String, (), String, napi::Status, false>>,
}

#[napi]
impl Emitter {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            attachCallback: None,
            detachCallback: None,
        }
    }

    #[napi]
    pub async unsafe fn start(&self) {
        let mut watch = nusb::watch_devices().unwrap();
        while let Some(event) = watch.next().await {
            match event {
                HotplugEvent::Connected(info) => {
                    if let Some(callback) = &self.attachCallback {
                        let device = UsbDevice::new(info);
                        callback.call(device, ThreadsafeFunctionCallMode::NonBlocking);
                    }
                }
                HotplugEvent::Disconnected(id) => {
                    if let Some(callback) = &self.detachCallback {
                        let handle = format!("{:?}", id);
                        callback.call(handle, ThreadsafeFunctionCallMode::NonBlocking);
                    }
                }
            }
        }
    }

    #[napi]
    pub fn addAttach(&mut self, callback: ThreadsafeFunction<UsbDevice, (), UsbDevice, napi::Status, false>) {
        self.attachCallback = Some(callback);
    }

    #[napi]
    pub fn removeAttach(&mut self, _callback: ThreadsafeFunction<UsbDevice, (), UsbDevice, napi::Status, false>) {
        self.attachCallback = None;
    }

    #[napi]
    pub fn addDetach(&mut self, callback: ThreadsafeFunction<String, (), String, napi::Status, false>) {
        self.detachCallback = Some(callback);
    }

    #[napi]
    pub fn removeDetach(&mut self, _callback: ThreadsafeFunction<String, (), String, napi::Status, false>) {
        self.detachCallback = None;
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
