const usb = require('./');
const devices = usb.listDevices();
console.log(devices);
console.log(usb.getVersion());
