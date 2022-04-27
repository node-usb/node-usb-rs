#![deny(clippy::all)]

use napi_derive::napi;

#[cfg(all(
  any(windows, unix),
  target_arch = "x86_64",
  not(target_env = "musl"),
  not(debug_assertions)
))]
#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[napi(object)]
pub struct Device {
  pub vendor: u32,
  pub product: u32,
}

#[napi]
pub fn list_devices() -> Vec<Device> {
  let mut vec = Vec::new();

  for device in rusb::devices().unwrap().iter() {
    let device_desc = device.device_descriptor().unwrap();

    vec.push(Device {
      vendor: device_desc.vendor_id() as u32,
      product: device_desc.product_id() as u32,
    });
  }

  vec
}
