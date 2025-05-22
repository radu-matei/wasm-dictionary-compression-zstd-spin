declare module 'component:compressor/compress' {
  
  export class Compressor {
    constructor(level: number, dict: string)
    addBytes(input: Uint8Array): Uint8Array;
    finish(): Uint8Array;
  }
}
