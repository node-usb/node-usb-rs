# Changelog

## [3.3.0] - 2026-11-08

### Changed
- Updated napi to v3.16.0 - [`105`](https://github.com/node-usb/node-usb-rs/pull/105) ([Rob Moran](https://github.com/thegecko))
- Updated nusb to v0.2.9 - [`106`](https://github.com/node-usb/node-usb-rs/pull/106) ([Rob Moran](https://github.com/thegecko))

### Fixed
- Fix filtering devices with multiple alternates - [`110`](https://github.com/node-usb/node-usb-rs/pull/110) ([Gonçalo MB](https://github.com/goncalomb))
- Don't fail or crash when the USB bus is missing - [`104`](https://github.com/node-usb/node-usb-rs/pull/104) ([Julian Waller](https://github.com/Julusian))
- Skip devices with unreadable descriptors - [`96`](https://github.com/node-usb/node-usb-rs/pull/96) ([Rob Moran](https://github.com/thegecko))
- Use nusb getters for device strings - [`86`](https://github.com/node-usb/node-usb-rs/pull/86) ([Rob Moran](https://github.com/thegecko))
- Fix active transfer close and release cleanup - [`85`](https://github.com/node-usb/node-usb-rs/pull/85) ([Rob Moran](https://github.com/thegecko))
- Fix transfer default timeout handling - [`84`](https://github.com/node-usb/node-usb-rs/pull/84) ([Rob Moran](https://github.com/thegecko))
- Fix endpoint transfer waits blocking pool starvation - [`83`](https://github.com/node-usb/node-usb-rs/pull/83) ([Rob Moran](https://github.com/thegecko))
- Fix controlTransferOut without data argument - [`82`](https://github.com/node-usb/node-usb-rs/pull/82) ([Rob Moran](https://github.com/thegecko))

## [3.2.1] - 2026-10-04

### Changed
- Updated napi to v3.13.0 - [`75`](https://github.com/node-usb/node-usb-rs/pull/75) ([Rob Moran](https://github.com/thegecko))

### Fixed
- Fixed concurrent access of endpoints - [`59`](https://github.com/node-usb/node-usb-rs/pull/59) ([Rob Moran](https://github.com/thegecko))

## [3.1.0] - 2026-08-22

### Fixed
- Guard against multiple hotplug watchers - [`36`](https://github.com/node-usb/node-usb-rs/pull/36) ([Thomas Kirkman-Wood](https://github.com/TKDubsta))

### Changed
- Updated nusb to v0.2.7 - [`43`](https://github.com/node-usb/node-usb-rs/pull/43) ([Rob Moran](https://github.com/thegecko))
- Removed CoffeeScript - [`29`](https://github.com/node-usb/node-usb-rs/pull/29) ([Rob Moran](https://github.com/thegecko))

## [3.0.1] - 2026-06-30

### Fixed
- Fixed device control transfers on macOS/Linux - [`16`](https://github.com/node-usb/node-usb-rs/pull/16) ([Rob Moran](https://github.com/thegecko))
- Fixed device hotplug watching to be more efficient - [`18`](https://github.com/node-usb/node-usb-rs/pull/18) ([Rob Moran](https://github.com/thegecko))

### Added
- Support for interrupt endpoints - [`4`](https://github.com/node-usb/node-usb-rs/pull/4) ([León](https://github.com/dressedinblack5))

### Changed
- Updated nusb to v0.2.4 - [`14`](https://github.com/node-usb/node-usb-rs/pull/14) ([Rob Moran](https://github.com/thegecko))
- Updated napi to v3.9.4 - [`19`](https://github.com/node-usb/node-usb-rs/pull/19) ([Rob Moran](https://github.com/thegecko))

## [3.0.0] - 2026-06-06

### Changed
- Initial release of the rust-based node-usb library
- Now uses the rust `nusb` library instead of `libusb`
- Dropped support for the `Legacy API` and exclusively uses the `WebUSB` API
