const usb = require('./');
const Worker = require('worker_threads').Worker;

const version = usb.getVersion();
console.log(`libusb ${version.major}.${version.minor}.${version.micro}`);

const devices = usb.listDevices();
console.log(devices.map(device => ({
    vendor: device.vendor,
    product: device.product
})));

for (let i = 0; i < 5; i ++) {
    const worker = new Worker('./worker.js')
    worker.on('message', serial => console.log(serial));
    worker.on('exit', code => console.log(code));
}
