import { getDeviceList, findBySerialNumber, USBDevice } from './index.js'
import { inspect } from 'util';

// create a JS subclass with extra methods
class WebUSBDevice extends USBDevice {
    async controlTransferIn(setup, length) {
        const res = await this.nativeControlTransferIn(setup, length);
        return {
            data: res ? new DataView(res.buffer) : null,
            status: res ? 'ok' : 'stall',
        };
    }

    async controlTransferOut(setup, data) {
        const res = await this.nativeControlTransferOut(setup, data);
        return {
            bytesWritten: res,
            status: res >= 0 ? 'ok' : 'stall',
        };
    }

    async transferIn(endpointNumber, length) {
        const res = await this.nativeTransferIn(endpointNumber, length);
        return {
            data: res ? new DataView(res.buffer) : null,
            status: res ? 'ok' : 'stall',
        };
    }

    async transferOut(endpointNumber, data) {
        const res = await this.nativeTransferOut(endpointNumber, data);
        return {
            bytesWritten: res,
            status: res >= 0 ? 'ok' : 'stall',
        };
    }
}

const toWebUSBDevice = (device) => Reflect.setPrototypeOf(device, WebUSBDevice.prototype);


const device2 = await findBySerialNumber('TEST_DEVICE');
toWebUSBDevice(device2);
console.log(inspect(device2, { showHidden: true, getters: true, depth: null }));
const devices = await getDeviceList();
console.log(inspect(devices, { showHidden: true, getters: true }));

let device = devices.find(d => d.vendorId === 0x59e3 && d.productId === 0x0a23);
toWebUSBDevice(device);
for (const dev of devices) {
    try {
        await device.open();
        console.log(`device opened: ${device.opened}`);
        await device.close();
        console.log(`device opened: ${device.opened}`);
    } catch (e) {
        console.error(e.message);
    }
}
try {
    await device.selectConfiguration(100);
} catch (e) {
    console.error(e.message);
}
await device.open();

try {
    await device.selectConfiguration(100);
} catch (e) {
    console.error(e.message);
}

await device.selectConfiguration(1);
await device.claimInterface(0)
console.log(inspect(device, { showHidden: true, getters: true, depth: null }));
console.log(`device opened: ${device.opened}`);

const b = Uint8Array.from(
    { length: 0x40 - 0x30 },
    (_, i) => 0x30 + i
).buffer;
console.log(b.byteLength)
console.log(b.toString());

let transferResult = await device.controlTransferOut({
    requestType: 'device',
    recipient: 'vendor',
    request: 0x81,
    value: 0,
    index: 0
}, b.Uint8Array)

console.log(transferResult.status);
console.log(transferResult.bytesWritten)

transferResult = await device.controlTransferIn({
    requestType: 'device',
    recipient: 'vendor',
    request: 0x81,
    value: 0,
    index: 0
}, 128)

console.log(transferResult.status);
console.log(transferResult.data.buffer.toString())

assert.equal(transferResult.status, 'ok')
assert.equal(transferResult.data.buffer.toString(), b.toString())

await device.close();
console.log(`device opened: ${device.opened}`);
