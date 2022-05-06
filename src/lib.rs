use std::time::Duration;
use napi_derive::napi;

#[napi(object)]
pub struct Version {
  pub major: u32,
  pub micro: u32,
  pub minor: u32,
  pub nano: u32,
}

#[napi(object)]
pub struct Device {
  pub vendor: u32,
  pub product: u32,
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
fn get_serial(vid: u32, pid: u32) -> String {
    let mut context = rusb::Context::new().unwrap();
    let (mut _device, mut handle) =
        open_device(&mut context, vid as u16, pid as u16).expect("Failed to open USB device");

    let serial = get_device_serial(&mut handle).unwrap();
    serial
}

fn open_device<T: rusb::UsbContext>(
    context: &mut T,
    vid: u16,
    pid: u16,
) -> Option<(rusb::Device<T>, rusb::DeviceHandle<T>)> {
    let devices = match context.devices() {
        Ok(d) => d,
        Err(_) => return None,
    };

    for device in devices.iter() {
        let device_desc = match device.device_descriptor() {
            Ok(d) => d,
            Err(_) => continue,
        };

        if device_desc.vendor_id() == vid && device_desc.product_id() == pid {
            match device.open() {
                Ok(handle) => return Some((device, handle)),
                Err(_) => continue,
            }
        }
    }

    None
}

fn get_device_serial<T: rusb::UsbContext>(handle: &mut rusb::DeviceHandle<T>) -> rusb::Result<String> {
    let device_desc = handle.device().device_descriptor()?;
    let timeout = Duration::from_secs(1);
    let languages = handle.read_languages(timeout)?;
    let language = languages[0];

    let serial = handle
        .read_serial_number_string(language, &device_desc, timeout)
        .unwrap_or("Not Found".to_string());
    
    Ok(serial)
}
