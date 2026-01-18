#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![deny(clippy::all)]

use napi::bindgen_prelude::*;
use napi_derive::napi;
use nusb::{descriptors::language_id::US_ENGLISH, MaybeFuture};
use std::time::Duration;

fn decode_version(version: u16) -> (u8, u8, u8) {
    let major: u8 = (version >> 8) as u8;
    let minor: u8 = ((version >> 4) & 0x0F) as u8;
    let sub: u8 = (version & 0x0F) as u8;
    (major, minor, sub)
}

#[napi(object)]
pub struct USBControlTransferParameters {
    #[napi(ts_type = "USBRequestType")]
    pub requestType: String,
    #[napi(ts_type = "USBRecipient")]
    pub recipient: String,
    pub request: u8,
    pub value: u16,
    pub index: u16,
}

#[napi(object)]
pub struct USBInTransferResult {
    #[napi(writable = false)]
    pub data: Option<Uint8Array>,
    #[napi(writable = false, ts_type = "USBTransferStatus")]
    pub status: String,
}

#[napi(object)]
pub struct USBOutTransferResult {
    #[napi(writable = false)]
    pub bytesWritten: u32,
    #[napi(writable = false, ts_type = "USBTransferStatus")]
    pub status: String,
}

#[napi]
pub struct USBDevice {
    device_info: nusb::DeviceInfo,
    device: Option<nusb::Device>,
    interfaces: Vec<Option<nusb::Interface>>,

    // pub id: DeviceId
    // pub port_chain: u8.
    // pub bus_id: &str,
    // pub device_address: u8,
    // pub speed: Option<Speed>,
    #[napi(writable = false)]
    pub vendorId: u16,
    #[napi(writable = false)]
    pub productId: u16,
    #[napi(writable = false)]
    pub deviceVersionMajor: u8,
    #[napi(writable = false)]
    pub deviceVersionMinor: u8,
    #[napi(writable = false)]
    pub deviceVersionSubminor: u8,
    #[napi(writable = false)]
    pub usbVersionMajor: u8,
    #[napi(writable = false)]
    pub usbVersionMinor: u8,
    #[napi(writable = false)]
    pub usbVersionSubminor: u8,
    #[napi(writable = false)]
    pub deviceClass: u8,
    #[napi(writable = false)]
    pub deviceSubclass: u8,
    #[napi(writable = false)]
    pub deviceProtocol: u8,
    #[napi(writable = false)]
    pub manufacturerName: Option<String>,
    #[napi(writable = false)]
    pub productName: Option<String>,
    #[napi(writable = false)]
    pub serialNumber: Option<String>,
    // pub interfaces: impl Iterator<Item = &InterfaceInfo>, --> configurations
    // readonly configuration: USBConfiguration | null;
    // readonly configurations: USBConfiguration[];

    // deviceinfo.interfaces ->interfaceinfo
    // device.configurations
    //    public configurations: USBConfiguration[] = [];
}

#[napi]
impl USBDevice {
    pub fn new(device_info: nusb::DeviceInfo) -> Self {
        let (deviceVersionMajor, deviceVersionMinor, deviceVersionSubminor) = decode_version(device_info.device_version());
        let (usbVersionMajor, usbVersionMinor, usbVersionSubminor) = decode_version(device_info.usb_version());

        Self {
            device_info: device_info.clone(),
            device: None,
            interfaces: vec![None; 256],
            vendorId: device_info.vendor_id(),
            productId: device_info.product_id(),
            deviceVersionMajor,
            deviceVersionMinor,
            deviceVersionSubminor,
            usbVersionMajor,
            usbVersionMinor,
            usbVersionSubminor,
            deviceClass: device_info.class(),
            deviceSubclass: device_info.subclass(),
            deviceProtocol: device_info.protocol(),
            manufacturerName: device_info.manufacturer_string().map(|s| s.to_string()),
            productName: device_info.product_string().map(|s| s.to_string()),
            serialNumber: device_info.serial_number().map(|s| s.to_string()),
        }
    }

    #[napi(getter)]
    pub fn opened(&self) -> bool {
        self.device.is_some()
    }

    /*
        public get configuration(): USBConfiguration | null {
      device.active_configuration
    }

     */

    #[napi]
    pub async unsafe fn open(&mut self) -> Result<()> {
        let device = self.device_info.open().wait().map_err(|e| napi::Error::from_reason(format!("open error: {e}")));
        self.device = device.ok();
        Ok(())
    }

    #[napi]
    pub async unsafe fn close(&mut self) -> Result<()> {
        self.device = None;
        Ok(())
    }

    #[napi]
    pub async unsafe fn forget(&mut self) -> Result<()> {
        self.close().await
    }

    #[napi]
    pub async fn reset(&self) -> Result<()> {
        match &self.device {
            Some(device) => device.reset().wait().map_err(|e| napi::Error::from_reason(format!("reset error: {e}"))),
            None => Err(napi::Error::from_reason("reset error: invalid state")),
        }
    }

    #[napi]
    pub async fn selectConfiguration(&self, configurationValue: u8) -> Result<()> {
        match &self.device {
            Some(device) => device
                .set_configuration(configurationValue)
                .wait()
                .map_err(|e| napi::Error::from_reason(format!("selectConfiguration error: {e}"))),
            None => Err(napi::Error::from_reason("selectConfiguration error: invalid state")),
        }
    }

    #[napi]
    pub async unsafe fn claimInterface(&mut self, interfaceNumber: u8) -> Result<()> {
        match &self.device {
            Some(device) => {
                let interface = device
                    .claim_interface(interfaceNumber)
                    .wait()
                    .map_err(|e| napi::Error::from_reason(format!("claimInterface error: {e}")))?;
                self.interfaces[interfaceNumber as usize] = Some(interface);
                Ok(())
            }
            None => Err(napi::Error::from_reason("claimInterface error: invalid state")),
        }
    }

    #[napi]
    pub async unsafe fn releaseInterface(&mut self, interfaceNumber: u8) -> Result<()> {
        match &self.device {
            Some(_device) => {
                self.interfaces[interfaceNumber as usize] = None;
                Ok(())
            }
            None => Err(napi::Error::from_reason("releaseInterface error: invalid state")),
        }
    }

    #[napi]
    pub async unsafe fn selectAlternateInterface(&mut self, interfaceNumber: u8, alternateSetting: u8) -> Result<()> {
        match &self.interfaces[interfaceNumber as usize] {
            Some(interface) => {
                interface
                    .set_alt_setting(alternateSetting)
                    .wait()
                    .map_err(|e| napi::Error::from_reason(format!("selectAlternateInterface error: {e}")))?;
                Ok(())
            }
            None => Err(napi::Error::from_reason("selectAlternateInterface error: invalid state")),
        }
    }

    #[napi]
    pub async fn controlTransferIn(&self, setup: USBControlTransferParameters, length: u16) -> Result<USBInTransferResult> {
        match &self.device {
            Some(device) => {
                let result = device
                    .control_in(
                        nusb::transfer::ControlIn {
                            control_type: match setup.requestType.as_str() {
                                "standard" => nusb::transfer::ControlType::Standard,
                                "class" => nusb::transfer::ControlType::Class,
                                "vendor" => nusb::transfer::ControlType::Vendor,
                                _ => nusb::transfer::ControlType::Standard,
                            },
                            recipient: match setup.recipient.as_str() {
                                "device" => nusb::transfer::Recipient::Device,
                                "interface" => nusb::transfer::Recipient::Interface,
                                "endpoint" => nusb::transfer::Recipient::Endpoint,
                                "other" => nusb::transfer::Recipient::Other,
                                _ => nusb::transfer::Recipient::Other,
                            },
                            request: setup.request,
                            value: setup.value,
                            index: setup.index,
                            length,
                        },
                        Duration::from_millis(100),
                    )
                    .wait()
                    .map_err(|e| napi::Error::from_reason(format!("controlTransferIn error: {e}")))?;
                Ok(USBInTransferResult {
                    data: Some(Uint8Array::from(result)),
                    status: "ok".to_string(),
                })
            }
            None => Err(napi::Error::from_reason("controlTransferIn error: invalid state")),
        }
    }

    #[napi]
    pub async fn controlTransferOut(&self, setup: USBControlTransferParameters, data: Option<Uint8Array>) -> Result<USBOutTransferResult> {
        match &self.device {
            Some(device) => {
                let bytes = data.map(|b| b.to_vec()).unwrap_or_default();
                device
                    .control_out(
                        nusb::transfer::ControlOut {
                            control_type: match setup.requestType.as_str() {
                                "standard" => nusb::transfer::ControlType::Standard,
                                "class" => nusb::transfer::ControlType::Class,
                                "vendor" => nusb::transfer::ControlType::Vendor,
                                _ => nusb::transfer::ControlType::Standard,
                            },
                            recipient: match setup.recipient.as_str() {
                                "device" => nusb::transfer::Recipient::Device,
                                "interface" => nusb::transfer::Recipient::Interface,
                                "endpoint" => nusb::transfer::Recipient::Endpoint,
                                "other" => nusb::transfer::Recipient::Other,
                                _ => nusb::transfer::Recipient::Other,
                            },
                            request: setup.request,
                            value: setup.value,
                            index: setup.index,
                            data: &bytes,
                        },
                        Duration::from_millis(100),
                    )
                    .wait()
                    .map_err(|e| napi::Error::from_reason(format!("controlTransferOut error: {e}")))?;
                Ok(USBOutTransferResult {
                    bytesWritten: bytes.len() as u32,
                    status: "ok".to_string(),
                })
            }
            None => Err(napi::Error::from_reason("controlTransferOut error: invalid state")),
        }
    }

    /* ENDPOINT */
    /*
        clearHalt(direction: USBDirection, endpointNumber: number): Promise<void>;
        public async clearHalt(direction: USBDirection, endpointNumber: number): Promise<void> {
          endpoint.clear_halt
        }

        transferIn(endpointNumber: number, length: number): Promise<USBInTransferResult>;
        transferOut(endpointNumber: number, data: BufferSource): Promise<USBOutTransferResult>;
        public async transferIn(endpointNumber: number, length: number): Promise<USBInTransferResult> {
          endpoint.reader
        }
        public async transferOut(endpointNumber: number, data: ArrayBuffer): Promise<USBOutTransferResult> {
          endpoint.writer
        }

        isochronousTransferIn(endpointNumber: number, packetLengths: number[]): Promise<USBIsochronousInTransferResult>;
        isochronousTransferOut(endpointNumber: number, data: BufferSource, packetLengths: number[], ): Promise<USBIsochronousOutTransferResult>;
        public async isochronousTransferIn(_endpointNumber: number, _packetLengths: number[]): Promise<USBIsochronousInTransferResult> {
            TransferType::Isochronous
            throw new Error('isochronousTransferIn error: method not implemented');
        }
        public async isochronousTransferOut(_endpointNumber: number, _data: BufferSource, _packetLengths: number[]): Promise<USBIsochronousOutTransferResult> {
            TransferType::Isochronous
            throw new Error('isochronousTransferOut error: method not implemented');
        }

    */
}

#[napi]
pub async fn list() -> Vec<USBDevice> {
    nusb::list_devices().wait().unwrap().map(|dev| USBDevice::new(dev)).collect()
}

#[napi]
pub async fn byidasync(vid: u16, pid: u16) -> USBDevice {
    let device = nusb::list_devices()
        .wait()
        .unwrap()
        .find(|dev| dev.vendor_id() == vid && dev.product_id() == pid)
        .expect("device not connected");

    USBDevice::new(device)
}

#[napi]
pub async fn serialbyidasync(vid: u16, pid: u16) -> String {
    let device = nusb::list_devices()
        .wait()
        .unwrap()
        .find(|dev| dev.vendor_id() == vid && dev.product_id() == pid)
        .expect("device not connected");

    let dev = match device.open().wait() {
        Ok(dev) => dev,
        Err(e) => {
            return format!("Failed to open device: {}", e);
        }
    };

    let timeout = Duration::from_millis(100);

    let dev_descriptor = dev.device_descriptor();

    let languages: Vec<u16> = dev.get_string_descriptor_supported_languages(timeout).wait().map(|i| i.collect()).unwrap_or_default();

    let language = languages.first().copied().unwrap_or(US_ENGLISH);

    if let Some(i_serial) = dev_descriptor.serial_number_string_index() {
        let s = dev.get_string_descriptor(i_serial, language, timeout).wait().unwrap();

        return format!("{s:?}");
    }

    return "  No Serial Number".to_string();
}

/*
impl From<nusb::Error> for napi::Error {
    fn from(e: nusb::Error) -> Self {
        napi::Error::Parse(e)
    }
}
*/
// watch_devices -> HotplugWatch
