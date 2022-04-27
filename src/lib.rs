#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

#[napi(object)]
pub struct Device {
  pub vendor: u32,
  pub product: u32,
}

#[napi(object)]
pub struct Version {
  pub major: u32,
  pub micro: u32,
  pub minor: u32,
  pub nano: u32,
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

#[napi]
fn get_version() -> Version {
  let version = rusb::version();
  let obj = Version {
    major: version.major() as u32,
    micro: version.micro() as u32,
    minor: version.minor() as u32,
    nano: version.nano() as u32,
  };

	obj
}
