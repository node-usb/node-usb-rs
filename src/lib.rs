#![allow(non_snake_case)]

mod webusb_device;

use futures_lite::StreamExt;
use napi::{bindgen_prelude::*, threadsafe_function::ThreadsafeFunction, threadsafe_function::ThreadsafeFunctionCallMode};
use napi_derive::napi;
use nusb::{hotplug::HotplugEvent, MaybeFuture};
use std::sync::Arc;
use tokio::sync::{watch, RwLock};
use webusb_device::UsbDevice;

struct Callbacks {
    attach: Option<ThreadsafeFunction<UsbDevice, (), UsbDevice, napi::Status, false>>,
    detach: Option<ThreadsafeFunction<String, (), String, napi::Status, false>>,
}

#[napi]
pub struct Emitter {
    callbacks: Arc<RwLock<Callbacks>>,
    listeners_tx: watch::Sender<bool>,
}

#[napi]
impl Emitter {
    #[napi(constructor)]
    pub fn new() -> Self {
        let callbacks = Arc::new(RwLock::new(Callbacks { attach: None, detach: None }));
        let (listeners_tx, _listeners_rx) = watch::channel(false);
        Self {
            callbacks,
            listeners_tx,
        }
    }

    #[napi]
    pub async fn init(&self) {
        let callbacks = self.callbacks.clone();
        let mut listeners_rx = self.listeners_tx.subscribe();

        tokio::spawn(async move {
            loop {
                // Async-wait until at least one listener is attached
                if listeners_rx.wait_for(|v| *v).await.is_err() {
                    return;
                }

                let mut watch_stream = nusb::watch_devices().unwrap();
                loop {
                    tokio::select! {
                        _ = listeners_rx.changed() => {
                            if !*listeners_rx.borrow() {
                                // No listeners attached, stop watching for device events
                                break;
                            }
                        }
                        ev = watch_stream.next() => {
                            match ev {
                                Some(HotplugEvent::Connected(info)) => {
                                    let guard = callbacks.read().await;
                                    if let Some(cb) = guard.attach.as_ref() {
                                        cb.call(UsbDevice::new(info), ThreadsafeFunctionCallMode::NonBlocking);
                                    }
                                }
                                Some(HotplugEvent::Disconnected(id)) => {
                                    let guard = callbacks.read().await;
                                    if let Some(cb) = guard.detach.as_ref() {
                                        cb.call(format!("{:?}", id), ThreadsafeFunctionCallMode::NonBlocking);
                                    }
                                }
                                None => break,
                            }
                        }
                    }
                }
            }
        });
    }

    #[napi]
    pub async unsafe fn addAttach(&mut self, callback: ThreadsafeFunction<UsbDevice, (), UsbDevice, napi::Status, false>) {
        { self.callbacks.write().await.attach = Some(callback); }
        self.publishState().await;
    }

    #[napi]
    pub async unsafe fn removeAttach(&mut self) {
        { self.callbacks.write().await.attach = None; }
        self.publishState().await;
    }

    #[napi]
    pub async unsafe fn addDetach(&mut self, callback: ThreadsafeFunction<String, (), String, napi::Status, false>) {
        { self.callbacks.write().await.detach = Some(callback); }
        self.publishState().await;
    }

    #[napi]
    pub async unsafe fn removeDetach(&mut self) {
        { self.callbacks.write().await.detach = None; }
        self.publishState().await;
    }

    async fn publishState(&self) {
        let listeners = {
            let cb = self.callbacks.read().await;
            cb.attach.is_some() || cb.detach.is_some()
        };
        let _ = self.listeners_tx.send(listeners);
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
