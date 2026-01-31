import { nativeGetDeviceList, nativeFindByIds, nativeFindBySerialNumber, UsbDevice, Emitter } from '../index.js'
import { EventEmitter } from 'events';

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

    /**
     * Optional flag to enable/disable automatic kernal driver detaching (defaults to true)
     */
    autoDetachKernelDriver?: boolean;
}

class NamedError extends Error {
    public constructor(message: string, name: string) {
        super(message);
        this.name = name;
    }
}

const augmentDevice = (device: UsbDevice): USBDevice => {

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

class WebUSB implements USB {

    protected nativeEmitter = new Emitter();
    protected emitter = new EventEmitter();
    protected authorisedDevices = new Set<USBDeviceFilter>();
    protected knownDevices: Map<string, USBDevice> = new Map();

    constructor(private options: USBOptions = {}) {
        const deviceConnectCallback = async (device: UsbDevice) => {
            const webDevice = augmentDevice(device);

            // When connected, emit an event if it is an allowed device
            if (webDevice && this.isAuthorisedDevice(webDevice)) {
                const event = {
                    type: 'connect',
                    device: webDevice
                };

                this.emitter.emit('connect', event);
            }
        };

        const deviceDisconnectCallback = async (handle: string) => {
            // When disconnected, emit an event if the device was a known allowed device
            if (this.knownDevices.has(handle)) {
                const webDevice = this.knownDevices.get(handle);
                if (webDevice && this.isAuthorisedDevice(webDevice)) {
                    const event = {
                        type: 'disconnect',
                        device: webDevice
                    };

                    this.emitter.emit('disconnect', event);
                }
            }
        };

        this.emitter.on('newListener', event => {
            const listenerCount = this.emitter.listenerCount(event);

            if (listenerCount !== 0) {
                return;
            }

            if (event === 'connect') {
                this.nativeEmitter.addAttach(deviceConnectCallback);
            } else if (event === 'disconnect') {
                // Ensure we know the current devices
                this.loadDevices();
                this.nativeEmitter.addDetach(deviceDisconnectCallback);
            }
        });

        this.emitter.on('removeListener', event => {
            const listenerCount = this.emitter.listenerCount(event);

            if (listenerCount !== 0) {
                return;
            }

            if (event === 'connect') {
                this.nativeEmitter.removeAttach(deviceConnectCallback);
            } else if (event === 'disconnect') {
                this.nativeEmitter.removeDetach(deviceDisconnectCallback);
            }
        });

        this.nativeEmitter.start();
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

    public addEventListener(type: 'connect' | 'disconnect', listener: (this: this, ev: USBConnectionEvent) => void): void;
    public addEventListener(type: 'connect' | 'disconnect', listener: EventListener): void;
    public addEventListener(type: string, listener: (ev: USBConnectionEvent) => void): void {
        this.emitter.addListener(type, listener);
    }

    public removeEventListener(type: 'connect' | 'disconnect', callback: (this: this, ev: USBConnectionEvent) => void): void;
    public removeEventListener(type: 'connect' | 'disconnect', callback: EventListener): void;
    public removeEventListener(type: string, callback: (this: this, ev: USBConnectionEvent) => void): void {
        this.emitter.removeListener(type, callback);
    }

    public dispatchEvent(_event: Event): boolean {
        // Don't dispatch from here
        return false;
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
        let nativeDevices = await nativeGetDeviceList();

        // Pre-filter devices
        nativeDevices = this.quickFilter(nativeDevices, preFilters);

        const devices: USBDevice[] = [];
        const refreshedKnownDevices = new Map<string, USBDevice>();


        for (const nativeDevice of nativeDevices) {
            const device = augmentDevice(nativeDevice);
            devices.push(device);
            refreshedKnownDevices.set(nativeDevice.handle, device);
        }

        this.knownDevices = refreshedKnownDevices;
        return devices;
    }

    // Undertake quick filter on devices before creating WebUSB devices if possible
    private quickFilter(devices: UsbDevice[], preFilters?: USBDeviceFilter[]): UsbDevice[] {
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
const getDeviceList = async (): Promise<USBDevice[]> => {
    const devices = await nativeGetDeviceList();
    return devices.map(device => augmentDevice(device));
};

/**
 * Convenience method to get the first device with the specified VID and PID, or `undefined` if no such device is present.
 * @param vid
 * @param pid
 */
const findByIds = async (vid: number, pid: number): Promise<USBDevice | undefined> => {
    const device = await nativeFindByIds(vid, pid);
    return device ? augmentDevice(device) : undefined;
};

/**
 * Convenience method to get the device with the specified serial number, or `undefined` if no such device is present.
 * @param serialNumber
 */
const findBySerialNumber = async (serialNumber: string): Promise<USBDevice | undefined> => {
    const device = await nativeFindBySerialNumber(serialNumber);
    return device ? augmentDevice(device) : undefined;
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
