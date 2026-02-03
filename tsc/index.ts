import { nativeGetDeviceList, nativeFindByIds, nativeFindBySerialNumber, UsbDevice, Emitter } from '../index.js'

const DEFAULT_TIMEOUT = 1000;

/**
 * USB Options
 */
interface USBOptions {
    /**
     * Optional `device found` callback function to allow the user to select a device
     */
    devicesFound?: (devices: USBDevice[]) => Promise<USBDevice | void>;

    /**
     * Optional array of preconfigured allowed devices
     */
    allowedDevices?: USBDeviceFilter[];

    /**
     * Optional flag to automatically allow all devices
     */
    allowAllDevices?: boolean;

    /**
     * Optional timeout (in milliseconds) to use for the device control transfers
     */
    deviceTimeout?: number;
}

class NamedError extends Error {
    public constructor(message: string, name: string) {
        super(message);
        this.name = name;
    }
}

class ConnectionEvent extends Event implements USBConnectionEvent {
    constructor(type: string, protected eventInitDict: USBConnectionEventInit) {
        super(type, eventInitDict);
    }
    public get device(): USBDevice {
        return this.eventInitDict.device;
    }
}

const augmentDevice = (device: UsbDevice, timeout: number): USBDevice => {

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

    Object.defineProperty(device, 'controlTransferIn', {
        enumerable: false,
        configurable: false,
        value: async function (setup: USBControlTransferParameters, length: number): Promise<USBInTransferResult> {
            const res = await this.nativeControlTransferIn(setup, timeout, length);
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
            const res = await this.nativeControlTransferOut(setup, timeout, toUint8Array(data));
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
            const res = await this.nativeTransferIn(endpointNumber, timeout, length);
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
            const res = await this.nativeTransferOut(endpointNumber, timeout, toUint8Array(data));
            return {
                bytesWritten: res,
                status: res >= 0 ? 'ok' : 'stall',
            };
        },
    });

    return device as unknown as USBDevice;
};

interface WebUSB {
    addEventListener(
        type: "connect" | "disconnect",
        listener: (this: this, ev: USBConnectionEvent) => any,
        useCapture?: boolean,
    ): void;
    addEventListener(
        type: string,
        listener: EventListenerOrEventListenerObject | null,
        options?: boolean | AddEventListenerOptions,
    ): void;
    removeEventListener(
        type: "connect" | "disconnect",
        callback: (this: this, ev: USBConnectionEvent) => any,
        useCapture?: boolean,
    ): void;
    removeEventListener(
        type: string,
        callback: EventListenerOrEventListenerObject | null,
        options?: EventListenerOptions | boolean,
    ): void;
}

class WebUSB extends EventTarget implements USB {

    protected nativeEmitter = new Emitter();
    protected authorisedDevices = new Set<USBDeviceFilter>();
    protected knownDevices: Map<string, USBDevice> = new Map();

    constructor(private options: USBOptions = {}) {
        super();

        const deviceConnectCallback = async (nativeDevice: UsbDevice) => {
            const device = augmentDevice(nativeDevice, this.options.deviceTimeout || DEFAULT_TIMEOUT);
            this.knownDevices.set(nativeDevice.handle, device);

            // When connected, emit an event if it is an allowed device
            if (device && this.isAuthorisedDevice(device)) {
                this.dispatchEvent(new ConnectionEvent('connect', { device }) );
            }
        };

        const deviceDisconnectCallback = async (handle: string) => {
            // When disconnected, emit an event if the device was a known allowed device
            if (this.knownDevices.has(handle)) {
                const device = this.knownDevices.get(handle)!;
                if (device && this.isAuthorisedDevice(device)) {
                    this.dispatchEvent(new ConnectionEvent('disconnect', { device }));
                }
                this.knownDevices.delete(handle);
            }
        };

        this.nativeEmitter.start();
        this.nativeEmitter.addAttach(deviceConnectCallback);
        this.nativeEmitter.addDetach(deviceDisconnectCallback);
        nativeGetDeviceList().then(devices => devices.forEach(device => this.knownDevices.set(device.handle, augmentDevice(device, this.options.deviceTimeout || DEFAULT_TIMEOUT))));
    }

    private _onconnect: ((ev: USBConnectionEvent) => void) | undefined;
    public set onconnect(fn: (ev: USBConnectionEvent) => void) {
        if (this._onconnect) {
            this.removeEventListener('connect', this._onconnect);
            this._onconnect = undefined;
        }

        if (fn) {
            this._onconnect = fn;
            this.addEventListener('connect', this._onconnect);
        }
    }

    private _ondisconnect: ((ev: USBConnectionEvent) => void) | undefined;
    public set ondisconnect(fn: (ev: USBConnectionEvent) => void) {
        if (this._ondisconnect) {
            this.removeEventListener('disconnect', this._ondisconnect);
            this._ondisconnect = undefined;
        }

        if (fn) {
            this._ondisconnect = fn;
            this.addEventListener('disconnect', this._ondisconnect);
        }
    }

    /**
     * Requests a single Web USB device
     * @param options The options to use when scanning
     * @returns Promise containing the selected device
     */
    public async requestDevice(options?: USBDeviceRequestOptions): Promise<USBDevice> {
        // Must have options
        if (!options) {
            throw new TypeError('requestDevice error: 1 argument required, but only 0 present');
        }

        // Options must be an object
        if (options.constructor !== {}.constructor) {
            throw new TypeError('requestDevice error: parameter 1 (options) is not an object');
        }

        // Must have a filter
        if (!options.filters) {
            throw new TypeError('requestDevice error: required member filters is undefined');
        }

        // Filter must be an array
        if (options.filters.constructor !== [].constructor) {
            throw new TypeError('requestDevice error: the provided value cannot be converted to a sequence');
        }

        // Check filters
        options.filters.forEach(filter => {
            // Protocol & Subclass
            if (filter.protocolCode && !filter.subclassCode) {
                throw new TypeError('requestDevice error: subclass code is required');
            }

            // Subclass & Class
            if (filter.subclassCode && !filter.classCode) {
                throw new TypeError('requestDevice error: class code is required');
            }
        });

        let devices = await this.loadDevices(options.filters);
        devices = devices.filter(device => this.filterDevice(device, options.filters));

        if (devices.length === 0) {
            throw new NamedError('Failed to execute \'requestDevice\' on \'USB\': No device selected.', 'NotFoundError');
        }

        try {
            // If no devicesFound function, select the first device found
            const device = this.options.devicesFound ? await this.options.devicesFound(devices) : devices[0];

            if (!device) {
                throw new NamedError('Failed to execute \'requestDevice\' on \'USB\': No device selected.', 'NotFoundError');
            }

            this.authorisedDevices.add({
                vendorId: device.vendorId,
                productId: device.productId,
                classCode: device.deviceClass,
                subclassCode: device.deviceSubclass,
                protocolCode: device.deviceProtocol,
                serialNumber: device.serialNumber || undefined
            });

            return device;
        } catch (error) {
            throw new NamedError('Failed to execute \'requestDevice\' on \'USB\': No device selected.', 'NotFoundError');
        }
    }

    /**
     * Gets all allowed Web USB devices which are connected
     * @returns Promise containing an array of devices
     */
    public async getDevices(): Promise<USBDevice[]> {
        const preFilters = this.options.allowAllDevices ? undefined : this.options.allowedDevices;

        // Refresh devices and filter for allowed ones
        const devices = await this.loadDevices(preFilters);

        return devices.filter(device => this.isAuthorisedDevice(device));
    }

    private async loadDevices(preFilters?: USBDeviceFilter[]): Promise<USBDevice[]> {
        let devices = await getDeviceList();

        // Pre-filter devices
        devices = this.quickFilter(devices, preFilters);
        return devices;
    }

    // Undertake quick filter on devices before creating WebUSB devices if possible
    private quickFilter(devices: USBDevice[], preFilters?: USBDeviceFilter[]): USBDevice[] {
        if (!preFilters || !preFilters.length) {
            return devices;
        }

        // Just pre-filter on vid/pid
        return devices.filter(device => preFilters.some(filter => {
            // Vendor
            if (filter.vendorId && filter.vendorId !== device.vendorId) return false;

            // Product
            if (filter.productId && filter.productId !== device.productId) return false;

            // Ignore Class, Subclass and Protocol as these need to check interfaces, too
            // Ignore serial number for node-usb as it requires device connection
            return true;
        }));
    }

    // Filter WebUSB devices
    private filterDevice(device: USBDevice, filters?: USBDeviceFilter[]): boolean {
        if (!filters || !filters.length) {
            return true;
        }

        return filters.some(filter => {
            // Vendor
            if (filter.vendorId && filter.vendorId !== device.vendorId) return false;

            // Product
            if (filter.productId && filter.productId !== device.productId) return false;

            // Class
            if (filter.classCode) {

                if (!device.configuration) {
                    return false;
                }

                // Interface Descriptors
                const match = device.configuration.interfaces.some(iface => {
                    // Class
                    if (filter.classCode && filter.classCode !== iface.alternate.interfaceClass) return false;

                    // Subclass
                    if (filter.subclassCode && filter.subclassCode !== iface.alternate.interfaceSubclass) return false;

                    // Protocol
                    if (filter.protocolCode && filter.protocolCode !== iface.alternate.interfaceProtocol) return false;

                    return true;
                });

                if (match) {
                    return true;
                }
            }

            // Class
            if (filter.classCode && filter.classCode !== device.deviceClass) return false;

            // Subclass
            if (filter.subclassCode && filter.subclassCode !== device.deviceSubclass) return false;

            // Protocol
            if (filter.protocolCode && filter.protocolCode !== device.deviceProtocol) return false;

            // Serial
            if (filter.serialNumber && filter.serialNumber !== device.serialNumber) return false;

            return true;
        });
    }

    // Check whether a device is authorised
    private isAuthorisedDevice(device: USBDevice): boolean {
        // All devices are authorised
        if (this.options.allowAllDevices) {
            return true;
        }

        // Check any allowed device filters
        if (this.options.allowedDevices && this.filterDevice(device, this.options.allowedDevices)) {
            return true;
        }

        // Check authorised devices
        return [...this.authorisedDevices.values()].some(authorised =>
            authorised.vendorId === device.vendorId
            && authorised.productId === device.productId
            && authorised.classCode === device.deviceClass
            && authorised.subclassCode === device.deviceSubclass
            && authorised.protocolCode === device.deviceProtocol
            && authorised.serialNumber === device.serialNumber
        );
    }
}

/**
 * Convenience method to get an array of all connected devices.
 */
const getDeviceList = async (timeout = DEFAULT_TIMEOUT): Promise<USBDevice[]> => {
    const devices = await nativeGetDeviceList();
    return devices.map(device => augmentDevice(device, timeout));
};

/**
 * Convenience method to get the first device with the specified VID and PID, or `undefined` if no such device is present.
 * @param vid
 * @param pid
 */
const findByIds = async (vid: number, pid: number, timeout = DEFAULT_TIMEOUT): Promise<USBDevice | undefined> => {
    const device = await nativeFindByIds(vid, pid);
    return device ? augmentDevice(device, timeout) : undefined;
};

/**
 * Convenience method to get the device with the specified serial number, or `undefined` if no such device is present.
 * @param serialNumber
 */
const findBySerialNumber = async (serialNumber: string, timeout = DEFAULT_TIMEOUT): Promise<USBDevice | undefined> => {
    const device = await nativeFindBySerialNumber(serialNumber);
    return device ? augmentDevice(device, timeout) : undefined;
};

const webusb = typeof navigator !== 'undefined' && navigator.usb ? navigator.usb : new WebUSB();

export {
    // Default WebUSB object (mimics navigator.usb)
    webusb,

    // Main object class
    WebUSB,

    // Types
    USBOptions,

    // Convenience methods
    getDeviceList,
    findByIds,
    findBySerialNumber,
};
