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

fn watch_task_guard(
    watch_task: &Mutex<Option<JoinHandle<()>>>,
) -> MutexGuard<'_, Option<JoinHandle<()>>> {
    watch_task
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[napi]
pub struct Emitter {
    callbacks: Arc<Mutex<Callbacks>>,
    watch_task: Arc<Mutex<Option<JoinHandle<()>>>>,
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
            watch_task: Arc::new(Mutex::new(None)),
        }
    }

    // addAttach and addDetach run at the same time, so one lock has to cover
    // both the check and the store. Release it in between and each starts a
    // watcher, which then reports every plug twice.
    fn start_watching(&self) -> Result<()> {
        let mut watch_task = watch_task_guard(&self.watch_task);

        if matches!(watch_task.as_ref(), Some(task) if !task.is_finished()) {
            return Ok(());
        }

        *watch_task = None;
        let callbacks = self.callbacks.clone();
        let mut watch_stream = match nusb::watch_devices() {
            Ok(watch_stream) => watch_stream,
            Err(e) => {
                return Err(napi::Error::from_reason(format!(
                    "watch devices error: {e}"
                )));
            }
        };

        *watch_task = Some(tokio::spawn(async move {
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

    fn stop_watching(&self) {
        // Keep the callbacks lock for the whole decision. Drop it after the
        // check and a listener registered in the gap would find its watcher
        // aborted here, leaving nothing watching.
        let callbacks = self.callbacks();
        let has_listeners = callbacks.attach.is_some() || callbacks.detach.is_some();

        if !has_listeners {
            if let Some(task) = watch_task_guard(&self.watch_task).take() {
                task.abort();
            }
        }
    }

    #[napi]
    pub async fn addAttach(
        &self,
        callback: ThreadsafeFunction<UsbDevice, (), UsbDevice, napi::Status, false>,
    ) -> Result<()> {
        {
            self.callbacks().attach = Some(callback);
        }
        self.start_watching()
    }

    #[napi]
    pub async fn removeAttach(&self) {
        {
            self.callbacks().attach = None;
        }
        self.stop_watching();
    }

    #[napi]
    pub async fn addDetach(
        &self,
        callback: ThreadsafeFunction<String, (), String, napi::Status, false>,
    ) -> Result<()> {
        {
            self.callbacks().detach = Some(callback);
        }
        self.start_watching()
    }

    #[napi]
    pub async fn removeDetach(&self) {
        {
            self.callbacks().detach = None;
        }
        self.stop_watching();
    }
}

impl Drop for Emitter {
    fn drop(&mut self) {
        if let Some(task) = watch_task_guard(&self.watch_task).take() {
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
