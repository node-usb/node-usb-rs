# USB Library for Node.JS

[![Build Status](https://github.com/node-usb/node-usb-rs/workflows/ci/badge.svg)](https://github.com/node-usb/node-usb-rs/actions)
[![npm](https://img.shields.io/npm/dm/usb.svg)](https://www.npmjs.com/package/usb)
[![Licence MIT](https://img.shields.io/badge/licence-MIT-blue.svg)](http://opensource.org/licenses/MIT)

Node.JS library for communicating with USB devices.

This is a complete rewrite in rust using [@kevinmehall](https://github.com/kevinmehall)'s excellent [nusb library](https://docs.rs/nusb/latest/nusb) and [napi-rs](https://napi.rs/).

# Prerequisites

[Node.js >= v12.22.0](https://nodejs.org), which includes `npm`.

## Windows

On Windows, if you get a `NOT_SUPPORTED` error when attempting to open your device, it's possible your device doesn't have a WinUSB driver to use.

You can install one using [Zadig](http://zadig.akeo.ie/).

## Linux

You may need to modify your udev and permission rules in order to access your desired device. Along the lines of:

```code
SUBSYSTEM=="usb", ATTR{idVendor}=="USB-VENDOR-ID", ATTR{idProduct}=="USB-PRODUCT-ID", MODE="0660", GROUP="GROUP-YOUR-USER-IS-IN"
```

# Installation

Native modules are bundled as separate optional packages, so installation should be as simple as installing the package.

With `npm`:

```bash
npm install usb
```

With `yarn`:

```bash
yarn add usb
```

# License
[MIT](LICENSE.md)

# Limitations
Does not support:

- Isochronous transfers

# Getting Started
Use the following examples to kickstart your development. Once you have a desired device, use the APIs below to interact with it.

## APIs
Since `v3.0.0`, the `node-usb` library supports the WebUSB API which follows the [WebUSB Specification](https://wicg.github.io/webusb/)

Convenience methods also exist to easily list or find devices.

Full auto-generated API documentation can be seen here:

https://node-usb.github.io/node-usb-rs/

## Electron
Please refer to the maintained example for using `node-usb` in electron:

https://github.com/node-usb/node-usb-example-electron

If using a packaging system for electron, ensure the `node-usb` library does not get recompiled as the correct binaries are already shipped with the package. For example, for [electron-builder](https://www.electron.build/), use these settings:

- buildDependenciesFromSource: true
- nodeGypRebuild: false
- npmRebuild: false

## Convenience Functions

### getDeviceList()
Return a list of `USB` objects for the USB devices attached to the system.

### findByIds(vid, pid)
Convenience method to get the first device with the specified VID and PID, or `undefined` if no such device is present.

### findBySerialNumber(serialNumber)
Convenience method to get the device with the specified serial number, or `undefined` if no such device is present.

## WebUSB

Please refer to the WebUSB specification which be found here:

https://wicg.github.io/webusb/

### Implementation Status

#### Architectures and Operating Systems

- i686-pc-windows-msvc
- x86_64-apple-darwin
- x86_64-pc-windows-msvc
- x86_64-unknown-linux-gnu
- x86_64-unknown-linux-musl
- aarch64-apple-darwin
- aarch64-pc-windows-msvc
- aarch64-unknown-linux-gnu
- aarch64-unknown-linux-musl
- armv7-unknown-linux-gnueabihf

#### USB

- [x] getDevices()
- [x] requestDevice()

#### USBDevice

- [x] usbVersionMajor
- [x] usbVersionMinor
- [x] usbVersionSubminor
- [x] deviceClass
- [x] deviceSubclass
- [x] deviceProtocol
- [x] vendorId
- [x] productId
- [x] deviceVersionMajor
- [x] deviceVersionMinor
- [x] deviceVersionSubminor
- [x] manufacturerName
- [x] productName
- [x] serialNumber
- [x] configuration
- [x] configurations
- [x] opened
- [x] open()
- [x] close()
- [x] selectConfiguration()
- [x] claimInterface()
- [x] releaseInterface()
- [x] selectAlternateInterface()
- [x] controlTransferIn()
- [x] controlTransferOut()
- [x] transferIn()
- [x] transferOut()
- [x] clearHalt()
- [x] reset()
- [x] forget()
- [ ] isochronousTransferIn()
- [ ] isochronousTransferOut()

#### Events

- [x] connect
- [x] disconnect

# Development
The library is based on native rust bindings wrapping the [nusb](https://docs.rs/nusb/latest/nusb) crate.

Ensure you have a working rust environment, instructions for setting this up are avalable at https://rust-lang.org/tools/install/

## Setup

```bash
git clone https://github.com/node-usb/node-usb-rs
```

## Building
The package can be built as follows:

```bash
npm install
npm run build:all
```

## Testing
To execute the unit tests, Run:

```bash
npm run full-test
```

Some tests require an [attached STM32F103 Microprocessor USB device with specific firmware](https://github.com/node-usb/node-usb-test-firmware).
