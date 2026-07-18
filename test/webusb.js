const assert = require('assert');
const usb = require('../dist').usb;
const webusb = require('../dist').webusb;
const WebUSB = require('../dist').WebUSB;

if (typeof gc === 'function') {
    // Running with --expose-gc, do a sweep between tests so valgrind blames the right one.
    afterEach(() => gc());
}

describe('usb Module', () => {
    it('should getDevices', async () => {
        const devs = await usb.getDevices();
        assert.ok(devs.length > 0, 'Demo device is not attached');
    });

    it('should findDeviceByIds', async () => {
        const dev = await usb.findDeviceByIds(0x59e3, 0x0a23);
        assert.ok(dev, 'Demo device is not attached');
    });

    it('should findDeviceBySerial', async () => {
        const dev = await usb.findDeviceBySerial('TEST_DEVICE');
        assert.ok(dev, 'Demo device is not attached');
    });
});

describe('WebUSB Module', () => {
    it('should describe basic constants', () => {
        assert.notEqual(webusb, undefined, 'webusb must be undefined');
    });
});

describe('allowedDevices', () => {
    it('should not list any devices by default', async () => {
        const l = await webusb.getDevices();
        assert.equal(l.length, 0);
    });

    it('should list allowed devices', async () => {
        const customWebusb = new WebUSB({ allowedDevices: [{ vendorId: 0x59e3 }] });
        const l = await customWebusb.getDevices();
        assert.equal(l.length, 1);
        assert.notEqual(l[0], undefined);
    });
});

describe('requestDevice', () => {
    it('should return a device', async () => {
        const device = await webusb.requestDevice({ filters: [{ vendorId: 0x59e3 }] });
        assert.notEqual(device, undefined);
    });
});

describe('getDevices', () => {
    it('should return one device', async () => {
        const device = await webusb.requestDevice({ filters: [{ vendorId: 0x59e3 }] });
        const l = await webusb.getDevices();
        assert.equal(l.length, 1);
        assert.notEqual(l[0], undefined);
        assert.deepEqual(l[0], device);
    });
});

describe('Device properties', () => {
    let device = null;

    before(async () => {
        device = await webusb.requestDevice({ filters: [{ vendorId: 0x59e3 }] });
    });

    it('should have usb version properties', () => {
        assert.equal(device.usbVersionMajor, 1);
        assert.equal(device.usbVersionMinor, 1);
        assert.equal(device.usbVersionSubminor, 0);
    });

    it('should have device version properties', () => {
        assert.equal(device.deviceVersionMajor, 1);
        assert.equal(device.deviceVersionMinor, 1);
        assert.equal(device.deviceVersionSubminor, 0);
    });

    it('should have class properties', () => {
        assert.equal(device.deviceClass, 0);
        assert.equal(device.deviceSubclass, 0);
    });

    it('should have protocol property', () => {
        assert.equal(device.deviceProtocol, 0);
    });

    it('should have vid/pid properties', () => {
        assert.equal(device.vendorId, 0x59e3);
        assert.equal(device.productId, 0x0a23);
    });

    it('should have a single configuration', () => {
        assert.equal(device.configurations.length, 1);
        assert.equal(device.configurations[0].configurationValue, 1);
    });

    it('should have a configuration property', () => {
        assert.notEqual(device.configuration, undefined);
    });

    it('should have a single interface', () => {
        assert.equal(device.configuration.interfaces.length, 1);
        assert.equal(device.configuration.interfaces[0].interfaceNumber, 0);
    });

    it('should have a single alternate', () => {
        assert.equal(device.configuration.interfaces[0].alternates.length, 1);
        assert.equal(device.configuration.interfaces[0].alternates[0].alternateSetting, 0);
    });

    it('should have extended properties', () => {
        assert.equal(typeof device.bus, 'string');
        assert.equal(typeof device.address, 'number');
        assert.equal(typeof device.speed, 'string');
        assert.ok(Array.isArray(device.ports));
    });
});

describe('String descriptors', () => {
    let device = null;

    before(async () => {
        device = await webusb.requestDevice({ filters: [{ vendorId: 0x59e3 }] });
    });

    it('gets serialNumber string', () => {
        assert.equal(device.serialNumber, 'TEST_DEVICE');
    });

    it('gets manufacturerName string', () => {
        assert.equal(device.manufacturerName, 'Nonolith Labs');
    });

    it('gets productName string', () => {
        assert.equal(device.productName, 'STM32F103 Test Device');
    });
});

describe('Device access', () => {
    let device = null;

    before(async () => {
        device = await webusb.requestDevice({ filters: [{ vendorId: 0x59e3 }] });
    });

    it('is not open', () => {
        assert.equal(device.opened, false);
    });

    it('is opens and closes', async () => {
        assert.equal(device.opened, false);
        await device.open();
        assert.equal(device.opened, true);
        await device.close();
        assert.equal(device.opened, false);
    });
});

describe('Configurations', () => {
    let device = null;

    before(async () => {
        device = await webusb.requestDevice({ filters: [{ vendorId: 0x59e3 }] });
        await device.open();
    });

    it('selects existing configuration', async () => {
        await assert.doesNotReject(device.selectConfiguration(1));
    });

    it('fails to select missing configuration', async () => {
        await assert.rejects(device.selectConfiguration(99));
    });

    after(async () => {
        await device.close();
    });
});

describe('Interfaces', () => {
    let device = null;

    before(async () => {
        device = await webusb.requestDevice({ filters: [{ vendorId: 0x59e3 }] });
        await device.open();
    });

    it('claims existing interface', async () => {
        await assert.doesNotReject(device.claimInterface(0));
    });

    it('fails to claim missing interface', async () => {
        await assert.rejects(device.claimInterface(99));
    });

    it('releases existing interface', async () => {
        await assert.doesNotReject(device.releaseInterface(0));
    });

    it('fails to release missing interface', async () => {
        await assert.rejects(device.releaseInterface(99));
    });

    if (process.platform === 'linux') {
        it('should fail to detach the kernel driver', async () => {
            await assert.rejects(device.detachKernelDriver(0));
        });

        it('should attach the kernel driver', async () => {
            await assert.doesNotReject(device.attachKernelDriver(0));
        });
    }

    after(async () => {
        await device.close();
    });
});

describe('Alternates', () => {
    let device = null;

    before(async () => {
        device = await webusb.requestDevice({ filters: [{ vendorId: 0x59e3 }] });
        await device.open();
        await device.claimInterface(0);
    });

    it('selects existing alternate', async () => {
        await assert.doesNotReject(device.selectAlternateInterface(0, 0));
    });

    after(async () => {
        await device.releaseInterface(0);
        if (process.platform !== 'win32') {
            await device.reset();
        }
        await device.close();
        await new Promise(resolve => setTimeout(resolve, 1000));
    });
});

describe('Control Transfers', () => {
    let device = null;
    const b1 = Uint8Array.from(Array.from({ length: 0x40 - 0x30 }, (_, i) => i + 0x30)).buffer;

    before(async () => {
        device = await webusb.requestDevice({ filters: [{ vendorId: 0x59e3 }] });
        await device.open();
        if (process.platform === 'win32') {
            await device.claimInterface(0);
        }
    });

    it('should control transfer OUT', async () => {
        const transferResult = await device.controlTransferOut({
            requestType: 'vendor',
            recipient: 'device',
            request: 0x81,
            value: 0,
            index: 0
        }, b1);

        assert.equal(transferResult.status, 'ok');
        assert.equal(transferResult.bytesWritten, b1.byteLength);
    });

    it('should control transfer IN', async () => {
        const transferResult = await device.controlTransferIn({
            requestType: 'vendor',
            recipient: 'device',
            request: 0x81,
            value: 0,
            index: 0
        }, 128);

        assert.equal(transferResult.status, 'ok');
        assert.equal(transferResult.data.byteLength, b1.byteLength);
        const resultBuffer = Buffer.from(transferResult.data.buffer, transferResult.data.byteOffset, transferResult.data.byteLength);
        const expectedBuffer = Buffer.from(b1, 0, b1.byteLength);
        assert(resultBuffer.equals(expectedBuffer));
    });

    after(async () => {
        if (process.platform === 'win32') {
            await device.releaseInterface(0);
        }
        await device.close();
    });
});

describe('Transfers', () => {
    let device = null;
    const b2 = Uint8Array.from(Array.from({ length: 0x42 - 0x32 }, (_, i) => i + 0x32)).buffer;

    before(async () => {
        device = await webusb.requestDevice({ filters: [{ vendorId: 0x59e3 }] });
        await device.open();
        await device.claimInterface(0);
    });

    it('should transfer OUT', async () => {
        const transferResult = await device.transferOut(4, b2);

        assert.equal(transferResult.status, 'ok');
        assert.equal(transferResult.bytesWritten, b2.byteLength);
    });

    it('should transfer IN', async () => {
        const transferResult = await device.transferIn(3, b2.byteLength);

        assert.equal(transferResult.status, 'ok');
        assert.equal(transferResult.data.byteLength, b2.byteLength);

        const resultBuffer = Buffer.from(transferResult.data.buffer, transferResult.data.byteOffset, transferResult.data.byteLength);
        const expectedBuffer = Buffer.from(b2, 0, b2.byteLength);
        assert(resultBuffer.equals(expectedBuffer));
    });

    after(async () => {
        await device.releaseInterface(0);
        await device.close();
    });
});

describe('Throwing Transfers', () => {
    let device = null;

    before(async () => {
        device = await webusb.requestDevice({ filters: [{ vendorId: 0x59e3 }] });
    });

    it('should fail control transfer unless opened', async () => {
        await assert.rejects(device.controlTransferIn({
            requestType: 'vendor',
            recipient: 'device',
            request: 0x81,
            value: 0,
            index: 0
        }, 128), 'The device must be opened first');
    });

    it('should fail transfer unless opened', async () => {
        await assert.rejects(device.transferIn(1, 64), 'The device must be opened first');
    });

    it('should fail transfer unless claimed', async () => {
        await device.open();
        await assert.rejects(device.transferIn(1, 64), 'The device must be claimed first');
    });

    after(async () => {
        await device.close();
    });
});

describe('WebUSB Hotplug', () => {
    it('should detect disconnect', done => {
        const fn = e => {
            assert.equal(e.device.serialNumber, 'TEST_DEVICE');
            webusb.removeEventListener('disconnect', fn);
            done();
        };

        webusb.addEventListener('disconnect', fn);
        console.log('\n--- DISCONNECT DEVICE ---\n');
    });

    it('should detect connect', done => {
        const fn = e => {
            assert.equal(e.device.serialNumber, 'TEST_DEVICE');
            webusb.removeEventListener('connect', fn);
            done();
        };

        webusb.addEventListener('connect', fn);
        console.log('\n--- CONNECT DEVICE ---\n');
    });
});
