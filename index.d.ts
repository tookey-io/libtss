/* tslint:disable */
/* eslint-disable */

/* auto-generated by NAPI-RS */

export interface EthersResult {
  result?: string
  error?: string
}
export function privateKeyToPublicKey(privateKey: string): EthersResult
export function privateKeyToEthersAddress(privateKey: string): EthersResult
export function signatureToEthersSignature(sign: string, data: string, chainId: number): EthersResult
export interface KeygenParams {
  roomId: string
  participantIndex: number
  participantsCount: number
  participantsThreshold: number
  relayAddress: string
  timeoutSeconds: number
}
export interface KeygenResult {
  key?: string
  error?: string
}
export function keygen(params: KeygenParams): Promise<KeygenResult>
export interface SignParams {
  roomId: string
  key: string
  data: string
  participantsIndexes: Array<number>
  relayAddress: string
  timeoutSeconds: number
}
export interface SignResult {
  result?: string
  error?: string
}
export function sign(params: SignParams): Promise<SignResult>
export function getVersion(): string
