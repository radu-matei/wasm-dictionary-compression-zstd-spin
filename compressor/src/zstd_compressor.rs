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

#[cfg(test)]
mod tests {
    use crate::bindings::exports::component::compressor::compress::GuestCompressor;
    use crate::zstd_compressor::ZstdCompressor;
    use std::fs::File;
    use std::io::Read;
    use zstd::Decoder;

    #[test]
    fn test_compress_and_finish() {
        let compressor = ZstdCompressor::new(0, "testdata/v1.dict".to_string());
        let data = b"Hello, world!".to_vec();

        let compressed_part = compressor.add_bytes(data.clone());
        assert!(
            !compressed_part.is_empty(),
            "Compressed part should not be empty"
        );

        let final_block = compressor.finish();
        assert!(!final_block.is_empty(), "Final block should not be empty");

        let combined = [compressed_part, final_block].concat();
        let mut dict_file = File::open("/testdata/v1.dict").unwrap();
        let mut dict = Vec::new();
        dict_file.read_to_end(&mut dict).unwrap();
        let mut decoder = Decoder::with_dictionary(combined.as_slice(), &dict).unwrap();
        let mut decompressed = vec![];
        decoder.read_to_end(&mut decompressed).unwrap();

        assert_eq!(
            data, decompressed,
            "Decompressed data should match original"
        );
    }

    #[test]
    fn test_multiple_add_bytes_calls() {
        let compressor = ZstdCompressor::new(0, "testdata/v1.dict".to_string());
        let chunk1 = compressor.add_bytes(b"Hello ".to_vec());
        let chunk2 = compressor.add_bytes(b"Rust ".to_vec());
        let chunk3 = compressor.add_bytes(b"World!".to_vec());
        assert!(!chunk1.is_empty() && !chunk2.is_empty() && !chunk3.is_empty());

        let final_block = compressor.finish();
        assert!(
            !final_block.is_empty(),
            "Final block should not be empty after finish"
        );
    }
}
