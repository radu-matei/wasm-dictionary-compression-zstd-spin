//@ts-ignore
import * as c from "component:compressor/compress";

/**
 * A TypeScript wrapper class around the underlying c.Compressor.
 */
export default class Compressor {
    private readonly internal: any;

    constructor(level: number, dict: string) {
        this.internal = new c.Compressor(level, dict);
    }

    addBytes(input: Uint8Array): Uint8Array {
        return this.internal.addBytes(input);
    }

    finish(): Uint8Array {
        return this.internal.finish();
    }
}
