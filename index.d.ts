/* eslint-disable */

export class ExternalObject<T> {
  readonly '': {
    readonly '': unique symbol
    [K: symbol]: T
  }
}
export interface Version {
  major: number
  minor: number
  micro: number
  nano: number
}
export function getVersion(): Version
export function listDevices(): Array<Device>
export function findByIds(vid: number, pid: number): Device | null
export class Device {
  vendor: number
  product: number
  open(): void
  readSerialNumberString(): string
  readManufacturerString(): string
  readProductString(): string
}
