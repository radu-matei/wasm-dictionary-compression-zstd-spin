#[allow(warnings)]
pub mod bindings;

mod zstd_compressor;

use zstd_compressor::ZstdCompressor;

bindings::export!(ZstdCompressor with_types_in bindings);
