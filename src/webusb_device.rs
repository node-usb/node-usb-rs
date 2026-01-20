#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use napi::bindgen_prelude::*;
use napi_derive::napi;
use nusb::{descriptors::language_id::US_ENGLISH, transfer::Bulk, MaybeFuture};
use std::io::{Read, Write};
use std::time::Duration;

fn decode_version(version: u16) -> (u8, u8, u8) {
    let major: u8 = (version >> 8) as u8;
    let minor: u8 = ((version >> 4) & 0x0F) as u8;
    let sub: u8 = (version & 0x0F) as u8;
    (major, minor, sub)
}

#[napi(object)]
pub struct USBIsochronousInTransferPacket {
    #[napi(writable = false)]
    pub data: Option<Uint8Array>,
    #[napi(writable = false, ts_type = "USBTransferStatus")]
    pub status: String,
}

#[napi(object)]
pub struct USBIsochronousInTransferResult {
    #[napi(writable = false)]
    pub data: Option<Uint8Array>,
    #[napi(writable = false)]
    pub packets: Vec<USBIsochronousInTransferPacket>,
}

#[napi(object)]
pub struct USBIsochronousOutTransferPacket {
    #[napi(writable = false)]
    pub bytesWritten: u32,
    #[napi(writable = false, ts_type = "USBTransferStatus")]
    pub status: String,
}

#[napi(object)]
pub struct USBIsochronousOutTransferResult {
    #[napi(writable = false)]
    pub packets: Vec<USBIsochronousOutTransferPacket>,
}

#[napi(object)]
pub struct USBEndpoint {
    #[napi(writable = false)]
    pub endpointNumber: u8,
    #[napi(writable = false, ts_type = "USBDirection")]
    pub direction: String,
    #[napi(writable = false, ts_type = "USBEndpointType", js_name = "type")]
    pub _type: String,
    #[napi(writable = false)]
    pub packetSize: u32,
}

impl USBEndpoint {
    pub fn new(endpoint: nusb::descriptors::EndpointDescriptor) -> Self {
        Self {
            endpointNumber: endpoint.address(),
            direction: if endpoint.direction() == nusb::transfer::Direction::In {
                "in".to_string()
            } else {
                "out".to_string()
            },
            _type: match endpoint.transfer_type() {
                nusb::descriptors::TransferType::Control => "control".to_string(),
                nusb::descriptors::TransferType::Isochronous => "isochronous".to_string(),
                nusb::descriptors::TransferType::Bulk => "bulk".to_string(),
                nusb::descriptors::TransferType::Interrupt => "interrupt".to_string(),
            },
            packetSize: endpoint.max_packet_size() as u32,
        }
    }
}

#[napi(object)]
pub struct USBAlternateInterface {
    #[napi(writable = false)]
    pub alternateSetting: u8,
    #[napi(writable = false)]
    pub interfaceClass: u8,
    #[napi(writable = false)]
    pub interfaceSubclass: u8,
    #[napi(writable = false)]
    pub interfaceProtocol: u8,
    #[napi(writable = false)]
    pub interfaceName: Option<String>,
    #[napi(writable = false)]
    pub endpoints: Vec<USBEndpoint>,
}

impl USBAlternateInterface {
    pub fn new(device: &nusb::Device, iface: nusb::descriptors::InterfaceDescriptor) -> Self {
        let interfaceName = match iface.string_index() {
            Some(desc_index) => Some(device.get_string_descriptor(desc_index, US_ENGLISH, Duration::from_millis(100)).wait().unwrap()),
            None => None,
        };

        Self {
            alternateSetting: iface.alternate_setting(),
            interfaceClass: iface.class(),
            interfaceSubclass: iface.subclass(),
            interfaceProtocol: iface.protocol(),
            interfaceName,
            endpoints: iface.endpoints().map(|endpoint| USBEndpoint::new(endpoint)).collect(),
        }
    }
}

#[napi(object)]
pub struct USBInterface {
    #[napi(writable = false)]
    pub interfaceNumber: u8,
    #[napi(writable = false)]
    pub claimed: bool,
    #[napi(writable = false)]
    pub alternate: USBAlternateInterface,
    #[napi(writable = false)]
    pub alternates: Vec<USBAlternateInterface>,
}

impl USBInterface {
    pub fn new(usb_device: &USBDevice, device: &nusb::Device, iface: nusb::descriptors::InterfaceDescriptors) -> Self {
        Self {
            interfaceNumber: iface.interface_number(),
            claimed: usb_device.interfaces[iface.interface_number() as usize].is_some(),
            alternate: USBAlternateInterface::new(&device, iface.first_alt_setting()),
            alternates: iface.alt_settings().map(|iface| USBAlternateInterface::new(&device, iface)).collect(),
        }
    }
}

#[napi(object)]
pub struct USBConfiguration {
    #[napi(writable = false)]
    pub configurationValue: u8,
    #[napi(writable = false)]
    pub configurationName: Option<String>,
    #[napi(writable = false)]
    pub interfaces: Vec<USBInterface>,
}

impl USBConfiguration {
    pub fn new(usb_device: &USBDevice, device: &nusb::Device, config: nusb::descriptors::ConfigurationDescriptor) -> Self {
        let configurationName = match config.string_index() {
            Some(desc_index) => Some(device.get_string_descriptor(desc_index, US_ENGLISH, Duration::from_millis(100)).wait().unwrap()),
            None => None,
        };

        let interfaces = config.interfaces().map(|iface| USBInterface::new(&usb_device, &device, iface)).collect();

        Self {
            configurationValue: config.configuration_value(),
            configurationName,
            interfaces,
        }
    }
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

    #[napi(getter)]
    pub unsafe fn configurations(&mut self) -> Vec<USBConfiguration> {
        let device = match self.device.as_ref() {
            Some(device) => device.clone(),
            None => self._open().unwrap(),
        };

        device.configurations().map(|config| USBConfiguration::new(&self, &device, config)).collect()
    }

    #[napi(getter)]
    pub unsafe fn configuration(&mut self) -> Option<USBConfiguration> {
        match &self.device {
            Some(device) => Some(USBConfiguration::new(&self, &device, device.active_configuration().unwrap())),
            None => None,
        }
    }

    unsafe fn _open(&mut self) -> Result<nusb::Device> {
        self.device_info.open().wait().map_err(|e| napi::Error::from_reason(format!("open error: {e}")))
    }

    #[napi]
    pub async unsafe fn open(&mut self) -> Result<()> {
        let device = self._open();
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

    fn get_endpoint<DIR: nusb::transfer::EndpointDirection>(&self, address: u8) -> Option<nusb::Endpoint<Bulk, DIR>> {
        for maybe_iface in &self.interfaces {
            let iface = match maybe_iface {
                Some(i) => i,
                None => continue,
            };

            for endpoint in iface.descriptor().unwrap().endpoints() {
                if endpoint.direction() == DIR::DIR && endpoint.address() == address {
                    return iface.endpoint::<Bulk, DIR>(address).ok();
                }
            }
        }

        None
    }

    #[napi]
    pub async fn clearHalt(&self, #[napi(ts_arg_type = "USBDirection")] direction: String, endpointNumber: u8) -> Result<()> {
        if direction == "in" {
            match self.get_endpoint::<nusb::transfer::In>(endpointNumber) {
                Some(mut endpoint) => {
                    endpoint.clear_halt().wait().map_err(|e| napi::Error::from_reason(format!("clearHalt error: {e}")))?;
                }
                None => {
                    return Err(napi::Error::from_reason("clearHalt error: endpoint not found"));
                }
            }
        } else {
            match self.get_endpoint::<nusb::transfer::Out>(endpointNumber) {
                Some(mut endpoint) => {
                    endpoint.clear_halt().wait().map_err(|e| napi::Error::from_reason(format!("clearHalt error: {e}")))?;
                }
                None => {
                    return Err(napi::Error::from_reason("clearHalt error: endpoint not found"));
                }
            }
        }

        Ok(())
    }

    #[napi]
    pub async fn transferIn(&self, endpointNumber: u8, length: u32) -> Result<USBInTransferResult> {
        match self.get_endpoint::<nusb::transfer::In>(endpointNumber) {
            Some(endpoint) => {
                let mut reader = endpoint.reader(4096);
                let mut buf = vec![0; length as usize];
                reader.read_exact(&mut buf)?;
                return Ok(USBInTransferResult {
                    data: Some(Uint8Array::from(buf)),
                    status: "ok".to_string(),
                });
            }
            None => {
                return Err(napi::Error::from_reason("transferIn error: endpoint not found"));
            }
        }
    }

    #[napi]
    pub async fn transferOut(&self, endpointNumber: u8, data: Uint8Array) -> Result<USBOutTransferResult> {
        match self.get_endpoint::<nusb::transfer::Out>(endpointNumber) {
            Some(endpoint) => {
                let mut writer = endpoint.writer(4096);
                writer.write_all(&data)?;
                writer.flush()?;
                return Ok(USBOutTransferResult {
                    bytesWritten: data.len() as u32,
                    status: "ok".to_string(),
                });
            }
            None => {
                return Err(napi::Error::from_reason("transferOut error: endpoint not found"));
            }
        }
    }

    #[napi]
    pub async fn isochronousTransferIn(&self, _endpointNumber: u8, _packetLengths: Vec<u32>) -> Result<USBIsochronousInTransferResult> {
        Err(napi::Error::from_reason("isochronousTransferIn error: method not implemented"))
    }

    #[napi]
    pub async fn isochronousTransferOut(&self, _endpointNumber: u8, _data: Uint8Array, _packetLengths: Vec<u32>) -> Result<USBIsochronousOutTransferResult> {
        Err(napi::Error::from_reason("isochronousTransferOut error: method not implemented"))
    }
}
