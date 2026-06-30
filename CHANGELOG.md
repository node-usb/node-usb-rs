# Changelog

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
