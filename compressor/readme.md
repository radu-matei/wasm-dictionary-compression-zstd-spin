# Wasm component for zstd compression

This directory contains a Wasm component that can be used to perform dictionary compression using zstd.

It implements the following WIT definition:

```wit
interface compress {
  resource compressor {
    /// The constructor for the compressor resource takes the compression level
    /// and the path to load the dictionary from disk.
    constructor(level: u8, dict: string);

    /// Receive bytes, write them into the encoder, flush, and
    /// return newly produced compressed bytes (since last call).
    add-bytes: func(input: list<u8>) -> list<u8>;

    /// Finish compression and return the final block of compressed bytes.
    /// After calling this, you cannot call `add_bytes` again.
    finish: func() -> list<u8>;
  }
}
```

This is implemented using the [zstd crate built in Rust](https://crates.io/crates/zstd).


To build, you need the [WASI SDK](https://github.com/WebAssembly/wasi-sdk):

```
# in the root of the repository
$ make compression
$ ls compressor/target/wasm32-wasip1/release/cmprsn_lib.wasm
605k compressor/target/wasm32-wasip1/release/cmprsn_lib.wasm

```

This generates a Wasm component in release mode. 
