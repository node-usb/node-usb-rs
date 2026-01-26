import { getDeviceList, findBySerialNumber, UsbDevice } from '../index.js'
import { inspect } from 'util';


const toUint8Array = (data: BufferSource): Uint8Array => {
    if (data instanceof ArrayBuffer) {
        return new Uint8Array(data);
    }

    // ArrayBufferView
    return new Uint8Array(
        data.buffer,
        data.byteOffset,
        data.byteLength
    );
};

const augmentDevice = (device: UsbDevice): USBDevice => {

    Object.defineProperty(device, 'controlTransferIn', {
        enumerable: false,
        configurable: false,
        value: async function (setup: USBControlTransferParameters, length: number): Promise<USBInTransferResult> {
            const res = await this.nativeControlTransferIn(setup, length);
            return {
                data: res ? new DataView(res.buffer) : undefined,
                status: res ? 'ok' : 'stall',
            };
        },
    });

    Object.defineProperty(device, 'controlTransferOut', {
        enumerable: false,
        configurable: false,
        value: async function (setup: USBControlTransferParameters, data: BufferSource): Promise<USBOutTransferResult> {
            const res = await this.nativeControlTransferOut(setup, toUint8Array(data));
            return {
                bytesWritten: res,
                status: res >= 0 ? 'ok' : 'stall',
            };
        },
    });

    Object.defineProperty(device, 'transferIn', {
        enumerable: false,
        configurable: false,
        value: async function (endpointNumber: number, length: number): Promise<USBInTransferResult> {
            const res = await this.nativeTransferIn(endpointNumber, length);
            return {
                data: res ? new DataView(res.buffer) : undefined,
                status: res ? 'ok' : 'stall',
            };
        },
    });

    Object.defineProperty(device, 'transferOut', {
        enumerable: false,
        configurable: false,
        value: async function (endpointNumber: number, data: BufferSource): Promise<USBOutTransferResult> {
            const res = await this.nativeTransferOut(endpointNumber, toUint8Array(data));
            return {
                bytesWritten: res,
                status: res >= 0 ? 'ok' : 'stall',
            };
        },
    });

    return device as unknown as USBDevice;
};

(async () => {
    const device2 = await findBySerialNumber('TEST_DEVICE');
    let webusbDev = augmentDevice(device2);
    console.log(inspect(webusbDev, { showHidden: true, getters: true, depth: null }));
    const devices = await getDeviceList();
    console.log(inspect(devices, { showHidden: true, getters: true }));

    let device = devices.find(d => d.vendorId === 0x59e3 && d.productId === 0x0a23);
    if (!device) {
        throw new Error('device not found');
    }

    webusbDev = augmentDevice(device);
    for (const dev of devices) {
        try {
            await dev.open();
            console.log(`device opened: ${dev.opened}`);
            await dev.close();
            console.log(`device opened: ${dev.opened}`);
        } catch (e) {
            console.error((e as Error).message);
        }
    }
    try {
        await webusbDev.selectConfiguration(100);
    } catch (e) {
        console.error((e as Error).message);
    }
    await webusbDev.open();

    try {
        await webusbDev.selectConfiguration(100);
    } catch (e) {
        console.error((e as Error).message);
    }

    await webusbDev.selectConfiguration(1);
    await webusbDev.claimInterface(0)
    console.log(inspect(webusbDev, { showHidden: true, getters: true, depth: null }));
    console.log(`device opened: ${webusbDev.opened}`);

    const b = Uint8Array.from(
        { length: 0x40 - 0x30 },
        (_, i) => 0x30 + i
    ).buffer;


    let outResult = await webusbDev.controlTransferOut({
        requestType: 'vendor',
        recipient: 'device',
        request: 0x81,
        value: 0,
        index: 0
    }, b)
    console.log(outResult.status);

    console.log(b.byteLength)
    console.log(b);
    console.log(outResult.bytesWritten);

    let inResult = await webusbDev.controlTransferIn({
        requestType: 'vendor',
        recipient: 'device',
        request: 0x81,
        value: 0,
        index: 0
    }, 128)

    console.log(inResult.status);
    console.log(Buffer.from(b).equals(Buffer.from(inResult.data!.buffer)));


    outResult = await webusbDev.transferOut(2, b)
    console.log(outResult.status);

    console.log(b.byteLength)
    console.log(b);
    console.log(outResult.bytesWritten);

    await new Promise(resolve => setTimeout(resolve, 1000));
   
    inResult = await webusbDev.transferIn(1, b.byteLength)
    console.log(inResult.status);
    console.log(Buffer.from(b).equals(Buffer.from(inResult.data!.buffer)));

    inResult = await webusbDev.transferIn(1, b.byteLength)
    console.log(inResult.status);
    console.log(Buffer.from(b).equals(Buffer.from(inResult.data!.buffer)));

    await webusbDev.close();
    console.log(`device opened: ${webusbDev.opened}`);
})();
