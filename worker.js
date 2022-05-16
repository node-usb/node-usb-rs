const usb = require('./');
const parentPort = require('worker_threads').parentPort

const device = usb.findByIds(0x59e3, 0x0a23);
device.open();
const serial = device.readSerialNumberString();
parentPort.postMessage(serial);
const vendor = device.readManufacturerString();
parentPort.postMessage(vendor);
const product = device.readProductString();
parentPort.postMessage(product);
