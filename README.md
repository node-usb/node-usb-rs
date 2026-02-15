# USB Library for Node.JS

[![Build Status](https://github.com/node-usb/node-usb-rs/workflows/ci/badge.svg)](https://github.com/node-usb/node-usb-rs/actions)
[![npm](https://img.shields.io/npm/dm/usb.svg)](https://www.npmjs.com/package/usb)
[![Licence MIT](https://img.shields.io/badge/licence-MIT-blue.svg)](http://opensource.org/licenses/MIT)

Node.JS library for communicating with USB devices.

This is a complete rewrite in rust using [@kevinmehall](https://github.com/kevinmehall)'s excellent [nusb library](https://docs.rs/nusb/latest/nusb) and [napi-rs](https://napi.rs/).

# License
[MIT](LICENSE.md)

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

# Getting Started

## Supported Architectures and Operating Systems

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

## Installation

Native modules are bundled as separate optional packages, so installation should be as simple as installing the package.

With `npm`:

```bash
npm install usb
```

With `yarn`:

```bash
yarn add usb
```

## Examples
Use the following examples to kickstart your development. Once you have a desired device, use the APIs below to interact with it.

### List all devices
```typescript
import { usb } from 'usb';

const devices = await usb.getDevices();

for (const device of devices) {
    console.log(device); // WebUSB device
}
```

### Find device by vid/pid
```typescript
import { usb } from 'usb';

const device = await usb.findDeviceByIds(0x59e3, 0x0a23);

if (device) {
    console.log(device); // WebUSB device
}
```

### Find device by SerialNumber
```typescript
import { usb } from 'usb';

const device = await usb.findDeviceBySerial('TEST_DEVICE');

if (device) {
    console.log(device); // WebUSB device
}
```

### Watch for connect/disconnect events
```typescript
import { usb } from 'usb';

usb.addEventListener('connect', (event) => {
    console.log('Device connected:', event.device.serialNumber);
});

usb.addEventListener('disconnect', (event) => {
    console.log('Device disconnected:', event.device.serialNumber);
});
```

### Use WebUSB approach to find a device
```typescript
import { webusb } from 'usb';

// Returns first matching device
const device = await webusb.requestDevice({
    filters: [{}]
})

console.log(device); // WebUSB device
```

### Use WebUSB approach to find a device with custom selection method
```typescript
import { WebUSB } from 'usb';

const customWebUSB = new WebUSB({
    // This function can return a promise which allows a UI to be displayed if required
    devicesFound: devices => devices.find(device => device.serialNumber === 'TEST_DEVICE')
});

// Returns device based on injected 'devicesFound' function
const device = await customWebUSB.requestDevice({
    filters: [{}]
})

console.log(device); // WebUSB device
```

### Electron
Please refer to the maintained example for using `node-usb` in electron:

https://github.com/node-usb/node-usb-example-electron

If using a packaging system for electron, ensure the `node-usb` library does not get recompiled as the correct binaries are already shipped with the package. For example, for [electron-builder](https://www.electron.build/), use these settings:

- buildDependenciesFromSource: true
- nodeGypRebuild: false
- npmRebuild: false

# APIs
Since `v3.0.0`, the `node-usb` API follows the WebUSB specification which can be found here:

https://wicg.github.io/webusb/

Two versions of the WebUSB API exist by default:

- `usb` - which exposes all functionality in an unrestricted manner (e.g. without needing to `requestDevice()` first)
- `webusb` - which follows the WebUSB specification exactly and requires the user to authorise devices via `requestDevice()` first.

You may also construct your own WebUSB (e.g. to specify a `requestDevice()` callback) using the exported `WebUSB` class.

Full auto-generated API documentation can be seen here:

https://node-usb.github.io/node-usb-rs/

## Implementation Status

### USB

#### WebUSB Features

- [x] getDevices()
- [x] requestDevice()

#### Extended Features

- [x] findDeviceByIds()
- [x] findDeviceBySerial()

### USBDevice

#### WebUSB Features

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

#### Extended Features

- [x] bus
- [x] address
- [x] ports
- [x] speed
- [x] detachKernelDriver() (Linux only)
- [x] attachKernelDriver() (Linux only)

### Events

- [x] connect
- [x] disconnect

## Extended Functions

This library extends the WebUSB specification to add further functionality and convenience

### findDeviceByIds(vid, pid)
Convenience method to get the first device with the specified VID and PID, or `undefined` if no such device is present.

### findDeviceBySerial(serialNumber)
Convenience method to get the device with the specified serial number, or `undefined` if no such device is present.

### detachKernelDriver(interfaceNumber) (Linux only)
Detaches the kernel driver from the interface.
You may need to execute this with elevated privileges.

### attachKernelDriver(interfaceNumber) (Linux only)
Re-attaches the kernel driver for the interface.

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
