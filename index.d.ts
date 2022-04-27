/* eslint-disable */

export class ExternalObject<T> {
  readonly '': {
    readonly '': unique symbol
    [K: symbol]: T
  }
}
export interface Device {
  vendor: number
  product: number
}
export function listDevices(): Array<Device>
