use napi_derive::napi;
use rusb::GlobalContext;
use std::time::Duration;

#[napi(object)]
struct Version {
    pub major: u32,
    pub minor: u32,
    pub micro: u32,
    pub nano: u32,
}

#[napi]
fn get_version() -> Version {
    let version = rusb::version();
    return Version {
        major: version.major() as u32,
        minor: version.minor() as u32,
        micro: version.micro() as u32,
        nano: version.nano() as u32,
    };
}

#[napi]
pub struct Device {
    device: rusb::Device<GlobalContext>,
    handle: Option<rusb::DeviceHandle<GlobalContext>>,
    pub vendor: u32,
    pub product: u32,
}

#[napi]
impl Device {
    #[napi]
    pub fn open(&mut self) {
        self.handle = self.device.open().ok();
    }

    #[napi]
    pub fn read_serial_number_string(&self) -> napi::Result<String> {
        let device_desc = self.device.device_descriptor().unwrap();
        let timeout = Duration::from_secs(1);
        let languages = self
            .handle
            .as_ref()
            .unwrap()
            .read_languages(timeout)
            .unwrap();
        let language = languages[0];

        let serial = self
            .handle
            .as_ref()
            .unwrap()
            .read_serial_number_string(language, &device_desc, timeout)
            .unwrap_or("Not Found".to_string());

        Ok(serial)
    }

    #[napi]
    pub fn read_manufacturer_string(&self) -> napi::Result<String> {
        let device_desc = self.device.device_descriptor().unwrap();
        let timeout = Duration::from_secs(1);
        let languages = self
            .handle
            .as_ref()
            .unwrap()
            .read_languages(timeout)
            .unwrap();
        let language = languages[0];

        let serial = self
            .handle
            .as_ref()
            .unwrap()
            .read_manufacturer_string(language, &device_desc, timeout)
            .unwrap_or("Not Found".to_string());

        Ok(serial)
    }

    #[napi]
    pub fn read_product_string(&self) -> napi::Result<String> {
        let device_desc = self.device.device_descriptor().unwrap();
        let timeout = Duration::from_secs(1);
        let languages = self
            .handle
            .as_ref()
            .unwrap()
            .read_languages(timeout)
            .unwrap();
        let language = languages[0];

        let serial = self
            .handle
            .as_ref()
            .unwrap()
            .read_product_string(language, &device_desc, timeout)
            .unwrap_or("Not Found".to_string());

        Ok(serial)
    }
}

#[napi]
fn list_devices() -> Vec<Device> {
    let mut vec = Vec::new();

    for device in rusb::devices().unwrap().iter() {
        let device_desc = device.device_descriptor().unwrap();

        vec.push(Device {
            handle: None,
            device: device,
            vendor: device_desc.vendor_id() as u32,
            product: device_desc.product_id() as u32,
        });
    }

    vec
}

#[napi]
fn find_by_ids(vid: u32, pid: u32) -> Option<Device> {
    for device in rusb::devices().unwrap().iter() {
        let device_desc = match device.device_descriptor() {
            Ok(d) => d,
            Err(_) => continue,
        };

        if device_desc.vendor_id() == vid as u16 && device_desc.product_id() == pid as u16 {
            return Some(Device {
                handle: None,
                device: device,
                vendor: device_desc.vendor_id() as u32,
                product: device_desc.product_id() as u32,
            });
        }
    }

    None
}
