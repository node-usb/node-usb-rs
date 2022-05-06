/* eslint-disable */

export class ExternalObject<T> {
  readonly '': {
    readonly '': unique symbol
    [K: symbol]: T
  }
}
export interface Version {
  major: number
  micro: number
  minor: number
  nano: number
}
export interface Device {
  vendor: number
  product: number
}
export function getVersion(): Version
export function listDevices(): Array<Device>
export function getSerial(vid: number, pid: number): string
