const rusb = require('./');
const devices = rusb.listDevices();
console.log(devices);
