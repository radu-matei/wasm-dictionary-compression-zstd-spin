use crate::bindings::exports::component::compressor::compress::{Guest, GuestCompressor};

use std::{
    cell::RefCell,
    io::{Cursor, Write},
};

use zstd::Encoder;

pub struct ZstdCompressor {
    inner: RefCell<CompressorInner>,
}

struct CompressorInner {
    encoder: Option<Encoder<'static, Cursor<Vec<u8>>>>,
    last_pos: usize,
}

impl GuestCompressor for ZstdCompressor {
    /// Constructor that takes a compression level and an optional dictionary path.
    fn new(level: u8, dict: String) -> Self {
        let dict_data = std::fs::read(&dict)
            .unwrap_or_else(|e| panic!("Failed to read dictionary file '{}': {}", dict, e));
        let encoder = Some(
            Encoder::with_dictionary(Cursor::new(Vec::new()), level as i32, &dict_data)
                .expect("failed to create zstd encoder with dictionary"),
        );

        Self {
            inner: RefCell::new(CompressorInner {
                encoder,
                last_pos: 0,
            }),
        }
    }

    /// Write the given `input` bytes to the encoder, flush, and return
    /// the newly produced compressed bytes since the last call.
    fn add_bytes(&self, input: Vec<u8>) -> Vec<u8> {
        let mut inner = self.inner.borrow_mut();

        {
            let encoder = inner.encoder.as_mut().expect("Compressor finished");
            encoder.write_all(&input).expect("write failed");
            encoder.flush().expect("flush failed");
        }

        let chunk = {
            let encoder = inner.encoder.as_ref().expect("Compressor finished");
            let buffer_ref = encoder.get_ref();
            let full_buffer = buffer_ref.get_ref();

            let last_pos = inner.last_pos;
            let buffer_len = full_buffer.len();

            let new_data = full_buffer[last_pos..].to_vec();
            inner.last_pos = buffer_len;

            new_data
        };

        chunk
    }

    /// Finish compression and return the final block of compressed bytes.
    /// After calling this, you cannot call `add_bytes` again.
    fn finish(&self) -> Vec<u8> {
        let mut inner = self.inner.borrow_mut();

        let encoder = match inner.encoder.take() {
            Some(enc) => enc,
            None => {
                panic!("Compressor already finished");
            }
        };

        let writer = encoder.finish().expect("failed to finish zstd encoding");
        let buffer = writer.into_inner(); // This is our Vec<u8>
        let chunk = buffer[inner.last_pos..].to_vec();
        inner.last_pos = buffer.len();

        chunk
    }
}

impl Guest for ZstdCompressor {
    type Compressor = ZstdCompressor;
}
