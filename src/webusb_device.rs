use napi::bindgen_prelude::*;
use napi_derive::napi;
use nusb::{descriptors::language_id::US_ENGLISH, transfer::Buffer, transfer::Bulk, MaybeFuture};
use std::time::Duration;

const ENDPOINT_NUMBER_MASK: u8 = 0x7f;

fn decode_version(version: u16) -> (u8, u8, u8) {
    let major: u8 = (version >> 8) as u8;
    let minor: u8 = ((version >> 4) & 0x0F) as u8;
    let sub: u8 = (version & 0x0F) as u8;
    (major, minor, sub)
}

#[napi(object, js_name = "USBEndpoint")]
pub struct UsbEndpoint {
    #[napi(writable = false)]
    pub endpointNumber: u8,
    #[napi(writable = false, ts_type = "USBDirection")]
    pub direction: String,
    #[napi(writable = false, ts_type = "USBEndpointType", js_name = "type")]
    pub _type: String,
    #[napi(writable = false)]
    pub packetSize: u32,
}

impl UsbEndpoint {
    pub fn new(endpoint: nusb::descriptors::EndpointDescriptor) -> Self {
        Self {
            endpointNumber: endpoint.address() & ENDPOINT_NUMBER_MASK,
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

#[napi(object, js_name = "USBAlternateInterface")]
pub struct UsbAlternateInterface {
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
    pub endpoints: Vec<UsbEndpoint>,
}

impl UsbAlternateInterface {
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
            endpoints: iface.endpoints().map(|endpoint| UsbEndpoint::new(endpoint)).collect(),
        }
    }
}

#[napi(object, js_name = "USBInterface")]
pub struct UsbInterface {
    #[napi(writable = false)]
    pub interfaceNumber: u8,
    #[napi(writable = false)]
    pub claimed: bool,
    #[napi(writable = false)]
    pub alternate: UsbAlternateInterface,
    #[napi(writable = false)]
    pub alternates: Vec<UsbAlternateInterface>,
}

impl UsbInterface {
    pub fn new(usb_device: &UsbDevice, device: &nusb::Device, iface: nusb::descriptors::InterfaceDescriptors) -> Self {
        Self {
            interfaceNumber: iface.interface_number(),
            claimed: usb_device.interfaces[iface.interface_number() as usize].is_some(),
            alternate: UsbAlternateInterface::new(&device, iface.first_alt_setting()),
            alternates: iface.alt_settings().map(|iface| UsbAlternateInterface::new(&device, iface)).collect(),
        }
    }
}

#[napi(object, js_name = "USBConfiguration")]
pub struct UsbConfiguration {
    #[napi(writable = false)]
    pub configurationValue: u8,
    #[napi(writable = false)]
    pub configurationName: Option<String>,
    #[napi(writable = false)]
    pub interfaces: Vec<UsbInterface>,
}

impl UsbConfiguration {
    pub fn new(usb_device: &UsbDevice, device: &nusb::Device, config: nusb::descriptors::ConfigurationDescriptor) -> Self {
        let configurationName = match config.string_index() {
            Some(desc_index) => Some(device.get_string_descriptor(desc_index, US_ENGLISH, Duration::from_millis(100)).wait().unwrap()),
            None => None,
        };

        let interfaces = config.interfaces().map(|iface| UsbInterface::new(&usb_device, &device, iface)).collect();

        Self {
            configurationValue: config.configuration_value(),
            configurationName,
            interfaces,
        }
    }
}

#[napi(object, js_name = "USBControlTransferParameters")]
pub struct UsbControlTransferParameters {
    #[napi(ts_type = "USBRequestType")]
    pub requestType: String,
    #[napi(ts_type = "USBRecipient")]
    pub recipient: String,
    pub request: u8,
    pub value: u16,
    pub index: u16,
}

#[napi]
pub struct UsbDevice {
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
impl UsbDevice {
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

    #[napi(getter, enumerable = false, configurable = false)]
    pub fn handle(&self) -> String {
        format!("{:?}", self.device_info.id())
    }

    #[napi(getter)]
    pub fn opened(&self) -> bool {
        self.device.is_some()
    }

    #[napi(getter)]
    pub unsafe fn configurations(&mut self) -> Vec<UsbConfiguration> {
        let device = match self.device.as_ref() {
            Some(device) => device.clone(),
            None => self._open().unwrap(),
        };

        device.configurations().map(|config| UsbConfiguration::new(&self, &device, config)).collect()
    }

    #[napi(getter)]
    pub unsafe fn configuration(&mut self) -> Option<UsbConfiguration> {
        match &self.device {
            Some(device) => Some(UsbConfiguration::new(&self, &device, device.active_configuration().unwrap())),
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

    #[napi(js_name = "nativeControlTransferIn")]
    pub async fn controlTransferIn(&self, setup: UsbControlTransferParameters, length: u16) -> Result<Option<Uint8Array>> {
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
                Ok(Some(Uint8Array::from(result)))
            }
            None => Err(napi::Error::from_reason("controlTransferIn error: invalid state")),
        }
    }

    #[napi(js_name = "nativeControlTransferOut")]
    pub async fn controlTransferOut(&self, setup: UsbControlTransferParameters, data: Option<Uint8Array>) -> Result<u32> {
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
                Ok(bytes.len() as u32)
            }
            None => Err(napi::Error::from_reason("controlTransferOut error: invalid state")),
        }
    }

    #[napi(js_name = "nativeTransferIn")]
    pub async fn transferIn(&self, endpointNumber: u8, length: u32) -> Result<Option<Uint8Array>> {
        match self.get_endpoint::<nusb::transfer::In>(endpointNumber) {
            Some(mut endpoint) => {
                let packet_size = endpoint.max_packet_size();
                let req = (((length as usize) + packet_size - 1) / packet_size) * packet_size;
                let buf = Buffer::new(req);
                let completion = endpoint.transfer_blocking(buf, Duration::from_millis(100));
                completion.status.map_err(|e| napi::Error::from_reason(format!("transferIn error: {e:?}")))?;
                let mut v = completion.buffer.into_vec();
                v.truncate(completion.actual_len.min(length as usize));
                return Ok(Some(Uint8Array::from(v)));
            }
            None => {
                return Err(napi::Error::from_reason("transferIn error: endpoint not found"));
            }
        }
    }

    #[napi(js_name = "nativeTransferOut")]
    pub async fn transferOut(&self, endpointNumber: u8, data: Uint8Array) -> Result<u32> {
        match self.get_endpoint::<nusb::transfer::Out>(endpointNumber) {
            Some(mut endpoint) => {
                let mut buf = Buffer::new(data.len());
                buf.extend_from_slice(&data);
                let completion = endpoint.transfer_blocking(buf, Duration::from_millis(100));
                completion.status.map_err(|e| napi::Error::from_reason(format!("transferOut error: {e:?}")))?;
                return Ok(completion.actual_len as u32);
            }
            None => {
                return Err(napi::Error::from_reason("transferOut error: endpoint not found"));
            }
        }
    }

    #[napi(ts_return_type = "Promise<USBIsochronousInTransferResult>")]
    pub async fn isochronousTransferIn(&self, _endpointNumber: u8, _packetLengths: Vec<u32>) -> Result<()> {
        Err(napi::Error::from_reason("isochronousTransferIn error: method not implemented"))
    }

    #[napi(ts_return_type = "Promise<USBIsochronousOutTransferResult>")]
    pub async fn isochronousTransferOut(&self, _endpointNumber: u8, _data: Uint8Array, _packetLengths: Vec<u32>) -> Result<()> {
        Err(napi::Error::from_reason("isochronousTransferOut error: method not implemented"))
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

    fn get_endpoint<DIR: nusb::transfer::EndpointDirection>(&self, endpointNumber: u8) -> Option<nusb::Endpoint<Bulk, DIR>> {
        for maybe_iface in &self.interfaces {
            let iface = match maybe_iface {
                Some(i) => i,
                None => continue,
            };

            for endpoint in iface.descriptor().unwrap().endpoints() {
                if endpoint.direction() == DIR::DIR && (endpoint.address() & ENDPOINT_NUMBER_MASK) == endpointNumber {
                    return iface.endpoint::<Bulk, DIR>(endpoint.address()).ok();
                }
            }
        }

        None
    }
}
