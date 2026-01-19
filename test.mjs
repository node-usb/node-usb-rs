import { list } from './index.js'
import { inspect } from 'util';

const devices = await list();
console.log(inspect(devices, { showHidden: true, getters: true }));

let device = devices.find(d => d.vendorId === 0x59e3 && d.productId === 0x0a23);
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
await device.claimInterface(0)
console.log(inspect(device, { showHidden: true, getters: true, depth: null }));
try {
    await device.selectConfiguration(100);
} catch (e) {
    console.error(e.message);
}

await device.selectConfiguration(1);
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
