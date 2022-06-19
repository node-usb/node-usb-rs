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
    pub vendor_id: u32,
    pub product_id: u32,
    pub device_class: u32,
    pub device_subclass: u32,
    pub device_protocol: u32,
    pub usb_version_major: u32,
    pub usb_version_minor: u32,
    pub usb_version_subminor: u32,
    pub device_version_major: u32,
    pub device_version_minor: u32,
    pub device_version_subminor: u32
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

        let manufacturer = self
            .handle
            .as_ref()
            .unwrap()
            .read_manufacturer_string(language, &device_desc, timeout)
            .unwrap_or("Not Found".to_string());

        Ok(manufacturer)
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

        let product = self
            .handle
            .as_ref()
            .unwrap()
            .read_product_string(language, &device_desc, timeout)
            .unwrap_or("Not Found".to_string());

        Ok(product)
    }
}

#[napi]
fn list_devices() -> Vec<Device> {
    let mut vec = Vec::new();

    for device in rusb::devices().unwrap().iter() {
        let device_desc = device.device_descriptor().unwrap();
        let usb_version = device_desc.usb_version();
        let device_version = device_desc.device_version();

        vec.push(Device {
            handle: None,
            device: device,
            vendor_id: device_desc.vendor_id() as u32,
            product_id: device_desc.product_id() as u32,
            device_class: device_desc.class_code() as u32,
            device_subclass: device_desc.sub_class_code() as u32,
            device_protocol: device_desc.protocol_code() as u32,
            usb_version_major: usb_version.0 as u32,
            usb_version_minor: usb_version.1 as u32,
            usb_version_subminor: usb_version.2 as u32,
            device_version_major: device_version.0 as u32,
            device_version_minor: device_version.1 as u32,
            device_version_subminor: device_version.2 as u32,
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
            let usb_version = device_desc.usb_version();
            let device_version = device_desc.device_version();
    
            return Some(Device {
                handle: None,
                device: device,
                vendor_id: device_desc.vendor_id() as u32,
                product_id: device_desc.product_id() as u32,
                device_class: device_desc.class_code() as u32,
                device_subclass: device_desc.sub_class_code() as u32,
                device_protocol: device_desc.protocol_code() as u32,
                usb_version_major: usb_version.0 as u32,
                usb_version_minor: usb_version.1 as u32,
                usb_version_subminor: usb_version.2 as u32,
                device_version_major: device_version.0 as u32,
                device_version_minor: device_version.1 as u32,
                device_version_subminor: device_version.2 as u32,
            });
        }
    }

    None
}

/*
  readonly attribute octet usbVersionMajor;
  readonly attribute octet usbVersionMinor;
  readonly attribute octet usbVersionSubminor;
  
  readonly attribute octet deviceVersionMajor;
  readonly attribute octet deviceVersionMinor;
  readonly attribute octet deviceVersionSubminor;
  
  readonly attribute octet deviceClass;
  readonly attribute octet deviceSubclass;
  readonly attribute octet deviceProtocol;
  
  readonly attribute unsigned short vendorId;
  readonly attribute unsigned short productId;
  
  readonly attribute DOMString? manufacturerName;
  readonly attribute DOMString? productName;
  readonly attribute DOMString? serialNumber;
  
  ---

  readonly attribute USBConfiguration? configuration;
  
  readonly attribute FrozenArray<USBConfiguration> configurations;
  
  readonly attribute boolean opened;
  
  Promise<undefined> open();
  Promise<undefined> close();
  Promise<undefined> forget();
  
  Promise<undefined> selectConfiguration(octet configurationValue);
  Promise<undefined> claimInterface(octet interfaceNumber);
  Promise<undefined> releaseInterface(octet interfaceNumber);
  Promise<undefined> selectAlternateInterface(octet interfaceNumber, octet alternateSetting);
  Promise<USBInTransferResult> controlTransferIn(USBControlTransferParameters setup, unsigned short length);
  Promise<USBOutTransferResult> controlTransferOut(USBControlTransferParameters setup, optional BufferSource data);
  Promise<undefined> clearHalt(USBDirection direction, octet endpointNumber);
  Promise<USBInTransferResult> transferIn(octet endpointNumber, unsigned long length);
  Promise<USBOutTransferResult> transferOut(octet endpointNumber, BufferSource data);
  Promise<USBIsochronousInTransferResult> isochronousTransferIn(octet endpointNumber, sequence<unsigned long> packetLengths);
  Promise<USBIsochronousOutTransferResult> isochronousTransferOut(octet endpointNumber, BufferSource data, sequence<unsigned long> packetLengths);
  Promise<undefined> reset();
*/