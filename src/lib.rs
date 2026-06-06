#![allow(non_snake_case)]

mod webusb_device;

use futures_lite::StreamExt;
use napi::{
    bindgen_prelude::*, threadsafe_function::ThreadsafeFunction,
    threadsafe_function::ThreadsafeFunctionCallMode,
};
use napi_derive::napi;
use nusb::{hotplug::HotplugEvent, MaybeFuture};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::{watch, RwLock};
use webusb_device::{run_blocking, UsbDevice};

struct Callbacks {
    attach: Option<ThreadsafeFunction<UsbDevice, (), UsbDevice, napi::Status, false>>,
    detach: Option<ThreadsafeFunction<String, (), String, napi::Status, false>>,
}

#[napi]
pub struct Emitter {
    callbacks: Arc<RwLock<Callbacks>>,
    listeners_tx: watch::Sender<bool>,
    initialized: AtomicBool,
}

#[napi]
impl Emitter {
    #[napi(constructor)]
    pub fn new() -> Self {
        let callbacks = Arc::new(RwLock::new(Callbacks {
            attach: None,
            detach: None,
        }));
        let (listeners_tx, _listeners_rx) = watch::channel(false);
        Self {
            callbacks,
            listeners_tx,
            initialized: AtomicBool::new(false),
        }
    }

    #[napi]
    pub async fn init(&self) -> Result<()> {
        if self.initialized.swap(true, Ordering::AcqRel) {
            return Ok(());
        }

        let callbacks = self.callbacks.clone();
        let mut listeners_rx = self.listeners_tx.subscribe();

        tokio::spawn(async move {
            loop {
                // Async-wait until at least one listener is attached
                if listeners_rx.wait_for(|v| *v).await.is_err() {
                    return;
                }

                let mut watch_stream = match nusb::watch_devices() {
                    Ok(watch_stream) => watch_stream,
                    Err(_) => return,
                };
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

        Ok(())
    }

    #[napi]
    pub async unsafe fn addAttach(
        &mut self,
        callback: ThreadsafeFunction<UsbDevice, (), UsbDevice, napi::Status, false>,
    ) {
        {
            self.callbacks.write().await.attach = Some(callback);
        }
        self.publishState().await;
    }

    #[napi]
    pub async unsafe fn removeAttach(&mut self) {
        {
            self.callbacks.write().await.attach = None;
        }
        self.publishState().await;
    }

    #[napi]
    pub async unsafe fn addDetach(
        &mut self,
        callback: ThreadsafeFunction<String, (), String, napi::Status, false>,
    ) {
        {
            self.callbacks.write().await.detach = Some(callback);
        }
        self.publishState().await;
    }

    #[napi]
    pub async unsafe fn removeDetach(&mut self) {
        {
            self.callbacks.write().await.detach = None;
        }
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

async fn list_devices(error_prefix: &'static str) -> Result<Vec<nusb::DeviceInfo>> {
    run_blocking(move || {
        nusb::list_devices()
            .wait()
            .map(|devices| devices.collect::<Vec<_>>())
            .map_err(|e| format!("{error_prefix} error: {e}"))
    })
    .await
}

#[napi(js_name = "nativeGetDevices")]
pub async fn getDevices() -> Result<Vec<UsbDevice>> {
    let devices = list_devices("getDevices").await?;
    Ok(devices.into_iter().map(UsbDevice::new).collect())
}

#[napi(js_name = "nativeFindDeviceByIds")]
pub async fn findDeviceByIds(vendorId: u16, productId: u16) -> Result<Option<UsbDevice>> {
    let device = list_devices("findDeviceByIds")
        .await?
        .into_iter()
        .find(|dev| dev.vendor_id() == vendorId && dev.product_id() == productId);
    Ok(device.map(UsbDevice::new))
}

#[napi(js_name = "nativeFindDeviceBySerial")]
pub async fn findDeviceBySerial(serialNumber: String) -> Result<Option<UsbDevice>> {
    let device = list_devices("findDeviceBySerial")
        .await?
        .into_iter()
        .find(|dev| dev.serial_number() == Some(serialNumber.as_str()));
    Ok(device.map(UsbDevice::new))
}
