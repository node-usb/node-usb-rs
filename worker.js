const usb = require('./');
const parentPort = require('worker_threads').parentPort

const serial = usb.getSerial(0x59e3, 0x0a23);
parentPort.postMessage(serial);
