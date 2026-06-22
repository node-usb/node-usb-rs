use napi::bindgen_prelude::*;
use napi_derive::napi;
use nusb::{
    descriptors::language_id::US_ENGLISH, descriptors::TransferType, transfer::Buffer,
    transfer::Bulk, transfer::Interrupt, MaybeFuture,
};
use std::time::Duration;

const ENDPOINT_NUMBER_MASK: u8 = 0x7f;
const DESC_TIMEOUT: Duration = Duration::from_millis(100);

fn decode_version(version: u16) -> (u8, u8, u8) {
    let major: u8 = (version >> 8) as u8;
    let minor: u8 = ((version >> 4) & 0x0F) as u8;
    let sub: u8 = (version & 0x0F) as u8;
    (major, minor, sub)
}

fn get_string(device: &nusb::Device, index: Option<std::num::NonZeroU8>) -> Result<Option<String>> {
    match index {
        Some(desc_index) => device
            .get_string_descriptor(desc_index, US_ENGLISH, DESC_TIMEOUT)
            .wait()
            .map(Some)
            .map_err(|e| napi::Error::from_reason(format!("getString error: {e}"))),
        None => Ok(None),
    }
}

pub(crate) async fn run_blocking<T, F>(f: F) -> Result<T>
where
    T: Send + 'static,
    F: FnOnce() -> std::result::Result<T, String> + Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| napi::Error::from_reason(format!("blocking task error: {e}")))?
        .map_err(napi::Error::from_reason)
}

/// Enum wrapping either a Bulk or Interrupt endpoint.
/// Both implement BulkOrInterrupt and share identical method signatures,
/// but are different concrete types in Rust's type system.
enum AnyEndpoint<DIR: nusb::transfer::EndpointDirection> {
    Bulk(nusb::Endpoint<nusb::transfer::Bulk, DIR>),
    Interrupt(nusb::Endpoint<nusb::transfer::Interrupt, DIR>),
}

impl<DIR: nusb::transfer::EndpointDirection> AnyEndpoint<DIR> {
    fn max_packet_size(&self) -> usize {
        match self {
            AnyEndpoint::Bulk(ep) => ep.max_packet_size(),
            AnyEndpoint::Interrupt(ep) => ep.max_packet_size(),
        }
    }

    fn transfer_blocking(
        &mut self,
        buf: nusb::transfer::Buffer,
        timeout: Duration,
    ) -> nusb::transfer::Completion {
        match self {
            AnyEndpoint::Bulk(ep) => ep.transfer_blocking(buf, timeout),
            AnyEndpoint::Interrupt(ep) => ep.transfer_blocking(buf, timeout),
        }
    }

    fn clear_halt_blocking(&mut self) -> std::result::Result<(), nusb::Error> {
        match self {
            AnyEndpoint::Bulk(ep) => ep.clear_halt().wait(),
            AnyEndpoint::Interrupt(ep) => ep.clear_halt().wait(),
        }
    }
}

#[napi(object)]
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

#[napi(object)]
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
    pub fn new(
        device: &nusb::Device,
        iface: nusb::descriptors::InterfaceDescriptor,
    ) -> Result<Self> {
        Ok(Self {
            alternateSetting: iface.alternate_setting(),
            interfaceClass: iface.class(),
            interfaceSubclass: iface.subclass(),
            interfaceProtocol: iface.protocol(),
            interfaceName: get_string(device, iface.string_index())?,
            endpoints: iface
                .endpoints()
                .map(|endpoint| UsbEndpoint::new(endpoint))
                .collect(),
        })
    }
}

#[napi(object)]
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
    pub fn new(
        usb_device: &UsbDevice,
        device: &nusb::Device,
        iface: nusb::descriptors::InterfaceDescriptors,
    ) -> Result<Self> {
        Ok(Self {
            interfaceNumber: iface.interface_number(),
            claimed: usb_device.interfaces[iface.interface_number() as usize].is_some(),
            alternate: UsbAlternateInterface::new(&device, iface.first_alt_setting())?,
            alternates: iface
                .alt_settings()
                .map(|iface| UsbAlternateInterface::new(&device, iface))
                .collect::<Result<Vec<_>>>()?,
        })
    }
}

#[napi(object)]
pub struct UsbConfiguration {
    #[napi(writable = false)]
    pub configurationValue: u8,
    #[napi(writable = false)]
    pub configurationName: Option<String>,
    #[napi(writable = false)]
    pub interfaces: Vec<UsbInterface>,
}

impl UsbConfiguration {
    pub fn new(
        usb_device: &UsbDevice,
        device: &nusb::Device,
        config: nusb::descriptors::ConfigurationDescriptor,
    ) -> Result<Self> {
        let interfaces = config
            .interfaces()
            .map(|iface| UsbInterface::new(&usb_device, &device, iface))
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            configurationValue: config.configuration_value(),
            configurationName: get_string(device, config.string_index())?,
            interfaces,
        })
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

    // Non WebUSB props
    #[napi(writable = false)]
    pub bus: String,
    #[napi(writable = false)]
    pub address: u8,
    #[napi(writable = false)]
    pub ports: Vec<u8>,
    #[napi(writable = false, ts_type = "USBSpeed")]
    pub speed: Option<String>,
}

#[napi]
impl UsbDevice {
    pub fn new(device_info: nusb::DeviceInfo) -> Self {
        let (deviceVersionMajor, deviceVersionMinor, deviceVersionSubminor) =
            decode_version(device_info.device_version());
        let (usbVersionMajor, usbVersionMinor, usbVersionSubminor) =
            decode_version(device_info.usb_version());

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
            bus: device_info.bus_id().to_string(),
            address: device_info.device_address(),
            ports: device_info.port_chain().to_vec(),
            speed: match device_info.speed() {
                Some(speed) => match speed {
                    nusb::Speed::Low => Some("low".to_string()),
                    nusb::Speed::Full => Some("full".to_string()),
                    nusb::Speed::High => Some("high".to_string()),
                    nusb::Speed::Super => Some("super".to_string()),
                    nusb::Speed::SuperPlus => Some("superPlus".to_string()),
                    _ => None,
                },
                None => None,
            },
        }
    }

    #[napi(getter)]
    pub fn handle(&self) -> String {
        format!("{:?}", self.device_info.id())
    }

    #[napi(getter)]
    pub unsafe fn manufacturerName(&mut self) -> Result<Option<String>> {
        match &self.device_info.manufacturer_string() {
            Some(str) => Ok(Some(str.to_string())),
            None => {
                let device = match self.device.as_ref() {
                    Some(device) => device.clone(),
                    None => self._open()?,
                };

                get_string(
                    &device,
                    device.device_descriptor().manufacturer_string_index(),
                )
            }
        }
    }

    #[napi(getter)]
    pub unsafe fn productName(&mut self) -> Result<Option<String>> {
        match &self.device_info.product_string() {
            Some(str) => Ok(Some(str.to_string())),
            None => {
                let device = match self.device.as_ref() {
                    Some(device) => device.clone(),
                    None => self._open()?,
                };

                get_string(&device, device.device_descriptor().product_string_index())
            }
        }
    }

    #[napi(getter)]
    pub unsafe fn serialNumber(&mut self) -> Result<Option<String>> {
        match &self.device_info.serial_number() {
            Some(str) => Ok(Some(str.to_string())),
            None => {
                let device = match self.device.as_ref() {
                    Some(device) => device.clone(),
                    None => self._open()?,
                };

                get_string(
                    &device,
                    device.device_descriptor().serial_number_string_index(),
                )
            }
        }
    }

    #[napi(getter)]
    pub fn opened(&self) -> bool {
        self.device.is_some()
    }

    #[napi(getter, ts_return_type = "USBConfiguration")]
    pub unsafe fn configuration(&mut self) -> Result<Option<UsbConfiguration>> {
        let device = match self.device.as_ref() {
            Some(device) => device.clone(),
            None => self._open()?,
        };

        let config = device
            .active_configuration()
            .map_err(|e| napi::Error::from_reason(format!("configuration error: {e}")))?;

        Ok(Some(UsbConfiguration::new(&self, &device, config)?))
    }

    #[napi(getter, ts_return_type = "Array<USBConfiguration>")]
    pub unsafe fn configurations(&mut self) -> Result<Vec<UsbConfiguration>> {
        let device = match self.device.as_ref() {
            Some(device) => device.clone(),
            None => self._open()?,
        };

        device
            .configurations()
            .map(|config| UsbConfiguration::new(&self, &device, config))
            .collect::<Result<Vec<_>>>()
    }

    unsafe fn _open(&mut self) -> Result<nusb::Device> {
        self.device_info
            .open()
            .wait()
            .map_err(|e| napi::Error::from_reason(format!("open error: {e}")))
    }

    #[napi]
    pub async unsafe fn open(&mut self) -> Result<()> {
        let device_info = self.device_info.clone();
        let device = run_blocking(move || {
            device_info
                .open()
                .wait()
                .map_err(|e| format!("open error: {e}"))
        })
        .await?;
        self.device = Some(device);
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
            Some(device) => {
                let device = device.clone();
                run_blocking(move || {
                    device
                        .reset()
                        .wait()
                        .map_err(|e| format!("reset error: {e}"))
                })
                .await
            }
            None => Err(napi::Error::from_reason("reset error: invalid state")),
        }
    }

    #[napi]
    pub async fn selectConfiguration(&self, configurationValue: u8) -> Result<()> {
        match &self.device {
            Some(device) => {
                let found = device
                    .configurations()
                    .any(|c| c.configuration_value() == configurationValue);
                if !found {
                    return Err(napi::Error::from_reason(
                        "selectConfiguration error: invalid configuration",
                    ));
                }

                #[cfg(windows)]
                {
                    // Unsupported, as per WebUSB spec on Windows
                    Ok(())
                }
                #[cfg(not(windows))]
                {
                    let device = device.clone();
                    run_blocking(move || {
                        device
                            .set_configuration(configurationValue)
                            .wait()
                            .map_err(|e| format!("selectConfiguration error: {e}"))
                    })
                    .await
                }
            }
            None => Err(napi::Error::from_reason(
                "selectConfiguration error: invalid state",
            )),
        }
    }

    #[napi]
    pub async unsafe fn claimInterface(&mut self, interfaceNumber: u8) -> Result<()> {
        match &self.device {
            Some(device) => {
                let device = device.clone();
                let interface = run_blocking(move || {
                    device
                        .claim_interface(interfaceNumber)
                        .wait()
                        .map_err(|e| format!("claimInterface error: {e}"))
                })
                .await?;
                self.interfaces[interfaceNumber as usize] = Some(interface);
                Ok(())
            }
            None => Err(napi::Error::from_reason(
                "claimInterface error: invalid state",
            )),
        }
    }

    #[napi]
    pub async unsafe fn releaseInterface(&mut self, interfaceNumber: u8) -> Result<()> {
        match &self.device {
            Some(_device) => match &self.interfaces[interfaceNumber as usize] {
                Some(_interface) => {
                    self.interfaces[interfaceNumber as usize] = None;
                    Ok(())
                }
                None => Err(napi::Error::from_reason(
                    "releaseInterface error: not claimed",
                )),
            },
            None => Err(napi::Error::from_reason(
                "releaseInterface error: invalid state",
            )),
        }
    }

    #[napi]
    pub async unsafe fn selectAlternateInterface(
        &mut self,
        interfaceNumber: u8,
        alternateSetting: u8,
    ) -> Result<()> {
        match &self.interfaces[interfaceNumber as usize] {
            Some(interface) => {
                let interface = interface.clone();
                run_blocking(move || {
                    interface
                        .set_alt_setting(alternateSetting)
                        .wait()
                        .map_err(|e| format!("selectAlternateInterface error: {e}"))
                })
                .await?;
                Ok(())
            }
            None => Err(napi::Error::from_reason(
                "selectAlternateInterface error: invalid state",
            )),
        }
    }

    #[napi(js_name = "nativeControlTransferIn")]
    pub async fn controlTransferIn(
        &self,
        setup: UsbControlTransferParameters,
        timeout: u32,
        length: u16,
    ) -> Result<Option<Uint8Array>> {
        let control_type = match setup.requestType.as_str() {
            "standard" => nusb::transfer::ControlType::Standard,
            "class" => nusb::transfer::ControlType::Class,
            "vendor" => nusb::transfer::ControlType::Vendor,
            _ => nusb::transfer::ControlType::Standard,
        };
        let recipient = match setup.recipient.as_str() {
            "device" => nusb::transfer::Recipient::Device,
            "interface" => nusb::transfer::Recipient::Interface,
            "endpoint" => nusb::transfer::Recipient::Endpoint,
            "other" => nusb::transfer::Recipient::Other,
            _ => nusb::transfer::Recipient::Other,
        };
        match self.get_interface(recipient, setup.index) {
            Some(interface) => {
                let result = run_blocking(move || {
                    interface
                        .control_in(
                            nusb::transfer::ControlIn {
                                control_type,
                                recipient,
                                request: setup.request,
                                value: setup.value,
                                index: setup.index,
                                length,
                            },
                            Duration::from_millis(timeout as u64),
                        )
                        .wait()
                        .map_err(|e| format!("controlTransferIn error: {e}"))
                })
                .await?;
                Ok(Some(Uint8Array::from(result)))
            }
            None => Err(napi::Error::from_reason(
                "controlTransferIn error: invalid state",
            )),
        }
    }

    #[napi(js_name = "nativeControlTransferOut")]
    pub async fn controlTransferOut(
        &self,
        setup: UsbControlTransferParameters,
        timeout: u32,
        data: Option<Uint8Array>,
    ) -> Result<u32> {
        let control_type = match setup.requestType.as_str() {
            "standard" => nusb::transfer::ControlType::Standard,
            "class" => nusb::transfer::ControlType::Class,
            "vendor" => nusb::transfer::ControlType::Vendor,
            _ => nusb::transfer::ControlType::Standard,
        };
        let recipient = match setup.recipient.as_str() {
            "device" => nusb::transfer::Recipient::Device,
            "interface" => nusb::transfer::Recipient::Interface,
            "endpoint" => nusb::transfer::Recipient::Endpoint,
            "other" => nusb::transfer::Recipient::Other,
            _ => nusb::transfer::Recipient::Other,
        };
        match self.get_interface(recipient, setup.index) {
            Some(interface) => {
                let bytes = data.map(|b| b.to_vec()).unwrap_or_default();
                let bytes_len = bytes.len();
                run_blocking(move || {
                    interface
                        .control_out(
                            nusb::transfer::ControlOut {
                                control_type,
                                recipient,
                                request: setup.request,
                                value: setup.value,
                                index: setup.index,
                                data: &bytes,
                            },
                            Duration::from_millis(timeout as u64),
                        )
                        .wait()
                        .map_err(|e| format!("controlTransferOut error: {e}"))
                })
                .await?;
                Ok(bytes_len as u32)
            }
            None => Err(napi::Error::from_reason(
                "controlTransferOut error: invalid state",
            )),
        }
    }

    #[napi(js_name = "nativeTransferIn")]
    pub async fn transferIn(
        &self,
        endpointNumber: u8,
        timeout: u32,
        length: u32,
    ) -> Result<Option<Uint8Array>> {
        match self.get_endpoint::<nusb::transfer::In>(endpointNumber) {
            Some(mut endpoint) => {
                let v = run_blocking(move || {
                    let packet_size = endpoint.max_packet_size();
                    let req = (((length as usize) + packet_size - 1) / packet_size) * packet_size;
                    let buf = Buffer::new(req);
                    let completion =
                        endpoint.transfer_blocking(buf, Duration::from_millis(timeout as u64));
                    completion
                        .status
                        .map_err(|e| format!("transferIn error: {e:?}"))?;
                    let mut v = completion.buffer.into_vec();
                    v.truncate(completion.actual_len.min(length as usize));
                    Ok(v)
                })
                .await?;
                Ok(Some(Uint8Array::from(v)))
            }
            None => {
                return Err(napi::Error::from_reason(
                    "transferIn error: endpoint not found",
                ));
            }
        }
    }

    #[napi(js_name = "nativeTransferOut")]
    pub async fn transferOut(
        &self,
        endpointNumber: u8,
        timeout: u32,
        data: Uint8Array,
    ) -> Result<u32> {
        match self.get_endpoint::<nusb::transfer::Out>(endpointNumber) {
            Some(mut endpoint) => {
                let data = data.to_vec();
                run_blocking(move || {
                    let mut buf = Buffer::new(data.len());
                    buf.extend_from_slice(&data);
                    let completion =
                        endpoint.transfer_blocking(buf, Duration::from_millis(timeout as u64));
                    completion
                        .status
                        .map_err(|e| format!("transferOut error: {e:?}"))?;
                    Ok(completion.actual_len as u32)
                })
                .await
            }
            None => {
                return Err(napi::Error::from_reason(
                    "transferOut error: endpoint not found",
                ));
            }
        }
    }

    #[napi(
        js_name = "nativeIsochronousTransferIn",
        ts_return_type = "Promise<USBIsochronousInTransferResult>"
    )]
    pub async fn isochronousTransferIn(
        &self,
        _endpointNumber: u8,
        _packetLengths: Vec<u32>,
        _timeout: u32,
    ) -> Result<()> {
        Err(napi::Error::from_reason(
            "isochronousTransferIn error: method not implemented",
        ))
    }

    #[napi(
        js_name = "nativeIsochronousTransferOut",
        ts_return_type = "Promise<USBIsochronousOutTransferResult>"
    )]
    pub async fn isochronousTransferOut(
        &self,
        _endpointNumber: u8,
        _data: Uint8Array,
        _packetLengths: Vec<u32>,
        _timeout: u32,
    ) -> Result<()> {
        Err(napi::Error::from_reason(
            "isochronousTransferOut error: method not implemented",
        ))
    }

    #[napi]
    pub async fn clearHalt(
        &self,
        #[napi(ts_arg_type = "USBDirection")] direction: String,
        endpointNumber: u8,
    ) -> Result<()> {
        if direction == "in" {
            match self.get_endpoint::<nusb::transfer::In>(endpointNumber) {
                Some(mut endpoint) => {
                    run_blocking(move || {
                        endpoint
                            .clear_halt_blocking()
                            .map_err(|e| format!("clearHalt error: {e}"))
                    })
                    .await?;
                }
                None => {
                    return Err(napi::Error::from_reason(
                        "clearHalt error: endpoint not found",
                    ));
                }
            }
        } else {
            match self.get_endpoint::<nusb::transfer::Out>(endpointNumber) {
                Some(mut endpoint) => {
                    run_blocking(move || {
                        endpoint
                            .clear_halt_blocking()
                            .map_err(|e| format!("clearHalt error: {e}"))
                    })
                    .await?;
                }
                None => {
                    return Err(napi::Error::from_reason(
                        "clearHalt error: endpoint not found",
                    ));
                }
            }
        }

        Ok(())
    }

    #[napi]
    pub async fn detachKernelDriver(&self, interfaceNumber: u8) -> Result<()> {
        match &self.device {
            Some(device) => {
                let device = device.clone();
                run_blocking(move || {
                    device
                        .detach_kernel_driver(interfaceNumber)
                        .map_err(|e| format!("detachKernelDriver error: {e}"))
                })
                .await
            }
            None => Err(napi::Error::from_reason(
                "detachKernelDriver error: invalid state",
            )),
        }
    }

    #[napi]
    pub async fn attachKernelDriver(&self, interfaceNumber: u8) -> Result<()> {
        match &self.device {
            Some(device) => {
                let device = device.clone();
                run_blocking(move || {
                    device
                        .attach_kernel_driver(interfaceNumber)
                        .map_err(|e| format!("attachKernelDriver error: {e}"))
                })
                .await
            }
            None => Err(napi::Error::from_reason(
                "attachKernelDriver error: invalid state",
            )),
        }
    }

    fn get_interface(
        &self,
        recipient: nusb::transfer::Recipient,
        index: u16,
    ) -> Option<nusb::Interface> {
        if recipient == nusb::transfer::Recipient::Interface {
            // If recipient is interface and index matches a claimed interface number use that interface
            if let Some(interface) = self
                .interfaces
                .get(index as usize)
                .and_then(|interface| interface.clone())
            {
                return Some(interface);
            }
        }
        if recipient == nusb::transfer::Recipient::Endpoint {
            // If recipient is endpoint and index matches an endpoint address use the interface that owns that endpoint
            for maybe_iface in &self.interfaces {
                let iface = match maybe_iface {
                    Some(i) => i,
                    None => continue,
                };

                let Some(descriptor) = iface.descriptor() else {
                    continue;
                };

                for endpoint in descriptor.endpoints() {
                    if endpoint.address() == index as u8 {
                        return Some(iface.clone());
                    }
                }
            }
        }

        // Return any claimed interface
        let maybe_iface = self.interfaces.iter().find_map(|x| x.clone());
        if maybe_iface.is_some() {
            return maybe_iface;
        }
        None
    }

    fn get_endpoint<DIR: nusb::transfer::EndpointDirection>(
        &self,
        endpointNumber: u8,
    ) -> Option<AnyEndpoint<DIR>> {
        for maybe_iface in &self.interfaces {
            let iface = match maybe_iface {
                Some(i) => i,
                None => continue,
            };

            let Some(descriptor) = iface.descriptor() else {
                continue;
            };

            for ep_desc in descriptor.endpoints() {
                if ep_desc.direction() == DIR::DIR
                    && (ep_desc.address() & ENDPOINT_NUMBER_MASK) == endpointNumber
                {
                    let addr = ep_desc.address();
                    return match ep_desc.transfer_type() {
                        TransferType::Bulk => iface
                            .endpoint::<Bulk, DIR>(addr)
                            .ok()
                            .map(AnyEndpoint::Bulk),
                        TransferType::Interrupt => iface
                            .endpoint::<Interrupt, DIR>(addr)
                            .ok()
                            .map(AnyEndpoint::Interrupt),
                        _ => None,
                    };
                }
            }
        }

        None
    }
}
