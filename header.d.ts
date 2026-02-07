export type USBSpeed = 'low' | 'full' | 'high' | 'super' | 'superPlus';

export declare interface UsbDevice extends USBDevice {
    /**
     * The USB bus the device is connected to (e.g. `1-1`, `2-3.4`, etc.)
     */
    bus: string;

    /**
     * The USB address the device is connected to
     */
    address: number;

    /**
     * The USB port numbers the device is connected through (e.g. `[1]`, `[3, 4]`, etc.)
     */
    ports: Array<number>

    /**
     * The USB speed of the device (e.g. `Low`, `Full`, `High`, `Super`, `SuperPlus` or `undefined` if unknown)
     */
    speed?: USBSpeed;

    /**
     * Detaches the kernel driver from the specified interface number (Linux only)
     * @param interfaceNumber 
     */
    detachKernelDriver(interfaceNumber: number): Promise<void>;

    /**
     * Attaches the kernel driver for the specified interface number (Linux only)
     * @param interfaceNumber 
     */
    attachKernelDriver(interfaceNumber: number): Promise<void>;
}
