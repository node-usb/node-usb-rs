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
export interface Version {
  major: number
  micro: number
  minor: number
  nano: number
}
export function listDevices(): Array<Device>
export function getVersion(): Version
