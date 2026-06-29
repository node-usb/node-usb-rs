#![allow(non_snake_case)]

mod webusb_device;

use futures_lite::StreamExt;
use napi::{
    bindgen_prelude::*, threadsafe_function::ThreadsafeFunction,
    threadsafe_function::ThreadsafeFunctionCallMode,
};
use napi_derive::napi;
use nusb::{hotplug::HotplugEvent, MaybeFuture};
use std::sync::{Arc, Mutex, MutexGuard};
use tokio::task::JoinHandle;
use webusb_device::{run_blocking, UsbDevice};

struct Callbacks {
    attach: Option<ThreadsafeFunction<UsbDevice, (), UsbDevice, napi::Status, false>>,
    detach: Option<ThreadsafeFunction<String, (), String, napi::Status, false>>,
}

fn callbacks_guard(callbacks: &Mutex<Callbacks>) -> MutexGuard<'_, Callbacks> {
    callbacks
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[napi]
pub struct Emitter {
    callbacks: Arc<Mutex<Callbacks>>,
    watch_task: Option<JoinHandle<()>>,
}

#[napi]
impl Emitter {
    fn callbacks(&self) -> MutexGuard<'_, Callbacks> {
        callbacks_guard(&self.callbacks)
    }

    #[napi(constructor)]
    pub fn new() -> Self {
        let callbacks = Arc::new(Mutex::new(Callbacks {
            attach: None,
            detach: None,
        }));
        Self {
            callbacks,
            watch_task: None,
        }
    }

    async fn start_watching(&mut self) -> Result<()> {
        if matches!(self.watch_task.as_ref(), Some(task) if !task.is_finished()) {
            return Ok(());
        }

        self.watch_task = None;
        let callbacks = self.callbacks.clone();
        let mut watch_stream = match nusb::watch_devices() {
            Ok(watch_stream) => watch_stream,
            Err(e) => {
                return Err(napi::Error::from_reason(format!(
                    "watch devices error: {e}"
                )));
            }
        };

        self.watch_task = Some(tokio::spawn(async move {
            while let Some(ev) = watch_stream.next().await {
                match ev {
                    HotplugEvent::Connected(info) => {
                        let guard = callbacks_guard(&callbacks);
                        if let Some(cb) = guard.attach.as_ref() {
                            cb.call(
                                UsbDevice::new(info),
                                ThreadsafeFunctionCallMode::NonBlocking,
                            );
                        }
                    }
                    HotplugEvent::Disconnected(id) => {
                        let guard = callbacks_guard(&callbacks);
                        if let Some(cb) = guard.detach.as_ref() {
                            cb.call(format!("{:?}", id), ThreadsafeFunctionCallMode::NonBlocking);
                        }
                    }
                }
            }
        }));

        Ok(())
    }

    async fn stop_watching(&mut self) {
        let has_listeners = {
            let cb = self.callbacks();
            cb.attach.is_some() || cb.detach.is_some()
        };

        if !has_listeners {
            if let Some(task) = self.watch_task.take() {
                task.abort();
            }
        }
    }

    #[napi]
    pub async unsafe fn addAttach(
        &mut self,
        callback: ThreadsafeFunction<UsbDevice, (), UsbDevice, napi::Status, false>,
    ) -> Result<()> {
        {
            self.callbacks().attach = Some(callback);
        }
        self.start_watching().await
    }

    #[napi]
    pub async unsafe fn removeAttach(&mut self) {
        {
            self.callbacks().attach = None;
        }
        self.stop_watching().await;
    }

    #[napi]
    pub async unsafe fn addDetach(
        &mut self,
        callback: ThreadsafeFunction<String, (), String, napi::Status, false>,
    ) -> Result<()> {
        {
            self.callbacks().detach = Some(callback);
        }
        self.start_watching().await
    }

    #[napi]
    pub async unsafe fn removeDetach(&mut self) {
        {
            self.callbacks().detach = None;
        }
        self.stop_watching().await;
    }
}

impl Drop for Emitter {
    fn drop(&mut self) {
        if let Some(task) = self.watch_task.take() {
            task.abort();
        }
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
