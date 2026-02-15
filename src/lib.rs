#![allow(non_snake_case)]

mod webusb_device;

use futures_lite::StreamExt;
use napi::{bindgen_prelude::*, threadsafe_function::ThreadsafeFunction, threadsafe_function::ThreadsafeFunctionCallMode};
use napi_derive::napi;
use nusb::{hotplug::HotplugEvent, MaybeFuture};
use webusb_device::UsbDevice;

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

#[napi(js_name = "nativeGetDevices")]
pub async fn getDevices() -> Result<Vec<UsbDevice>> {
    let devices = nusb::list_devices().wait().map_err(|e| napi::Error::from_reason(format!("getDevices error: {e}")))?;
    Ok(devices.map(UsbDevice::new).collect())
}

#[napi(js_name = "nativeFindDeviceByIds")]
pub async fn findDeviceByIds(vendorId: u16, productId: u16) -> Result<Option<UsbDevice>> {
    let mut devices = nusb::list_devices().wait().map_err(|e| napi::Error::from_reason(format!("findDeviceByIds error: {e}")))?;
    Ok(devices.find(|dev| dev.vendor_id() == vendorId && dev.product_id() == productId).map(UsbDevice::new))
}

#[napi(js_name = "nativeFindDeviceBySerial")]
pub async fn findDeviceBySerial(serialNumber: String) -> Result<Option<UsbDevice>> {
    let mut devices = nusb::list_devices()
        .wait()
        .map_err(|e| napi::Error::from_reason(format!("findDeviceBySerial error: {e}")))?;
    Ok(devices.find(|dev| dev.serial_number() == Some(serialNumber.as_str())).map(UsbDevice::new))
}
