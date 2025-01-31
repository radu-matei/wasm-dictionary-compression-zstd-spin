//@ts-ignore
import * as c from "component:compressor/compress";

/**
 * A TypeScript wrapper class around the underlying c.Compressor.
 * We implement the same methods declared in the Compressor interface.
 */
export default class ZstdCompressor {
    private readonly internal: any; // or unknown if you prefer

    constructor(level: number, dict: string) {
        // The actual constructor call to the underlying module:
        this.internal = new c.Compressor(level, dict);
    }

    addBytes(input: Uint8Array): Uint8Array {
        return this.internal.addBytes(input);
    }

    finish(): Uint8Array {
        return this.internal.finish();
    }
}
