#[allow(warnings)]
mod bindings;

use bindings::exports::component::compressor::compress::{Guest, GuestCompressor};

use std::{
    cell::{Cell, RefCell},
    io::Write,
};
use anyhow::Result;

use brotli::CompressorWriter;
use crate::bindings::exports::component::compressor::compress::{InputStream, OutputStream};

pub struct BrotliCompressor {
    writer: RefCell<Option<CompressorWriter<Vec<u8>>>>,
    offset: Cell<usize>,
}

impl BrotliCompressor {
    pub(crate) fn compress_stream(&self, input: &InputStream, output: &OutputStream) -> Result<()> {
        while let Some(chunk) = input.blocking_read(8192).ok() {
            let buf = self.add_bytes(chunk);
            output.blocking_write_and_flush(&buf)?;
        }

        let buf = self.finish();
        output.blocking_write_and_flush(&buf)?;
        Ok(())
    }
}

impl GuestCompressor for BrotliCompressor {
    fn new(level: u8, _dict_path: String) -> Self {
        let quality = level.min(11) as u32;
        let lgwin = 22u32;

        let writer = CompressorWriter::new(Vec::new(), 4096, quality, lgwin);

        Self {
            writer: RefCell::new(Some(writer)),
            offset: Cell::new(0),
        }
    }

    /// Add bytes to the stream, flush, and return newly generated compressed bytes
    fn add_bytes(&self, input: Vec<u8>) -> Vec<u8> {
        let mut writer_guard = self.writer.borrow_mut();
        let writer = writer_guard
            .as_mut()
            .expect("Compressor was already finished");

        writer.write_all(&input).expect("Brotli write failed");
        writer.flush().expect("Brotli flush failed");

        let buffer = writer.get_ref();
        let offset = self.offset.get();

        let new_chunk = buffer[offset..].to_vec();
        self.offset.set(buffer.len());

        new_chunk
    }

    /// Finalize the stream, return the last block
    fn finish(&self) -> Vec<u8> {
        let mut writer_guard = self.writer.borrow_mut();
        let mut writer = writer_guard.take().expect("Already finished");

        writer.flush().expect("Brotli flush failed");

        let compressed = writer.into_inner();
        let offset = self.offset.get();
        compressed[offset..].to_vec()
    }

    fn pipe_through(&self, input: &InputStream, output: &OutputStream) {
        self.compress_stream(input, output).unwrap();
    }
}

pub struct ZstdCompressor {
    encoder: RefCell<Option<zstd::Encoder<'static, Vec<u8>>>>,
    offset: Cell<usize>,
}

impl GuestCompressor for ZstdCompressor {
    /// The constructor for the compressor resource takes the compression level
    /// and the path to load the dictionary from disk.
    fn new(level: u8, _dict_path: String) -> Self {
        // let dict_data = if dict_path.is_empty() {
        //     Vec::new()
        // } else {
        //     let mut file = File::open(&dict_path)
        //         .unwrap_or_else(|e| panic!("Cannot open dict '{}': {}", dict_path, e));
        //     let mut buf = Vec::new();
        //     file.read_to_end(&mut buf)
        //         .expect("Could not read dict file");
        //     buf
        // };

        // let encoder = if dict_data.is_empty() {
        //     // No dictionary
        let encoder =
            zstd::Encoder::new(Vec::new(), level as i32).expect("Failed to create zstd encoder");
        // } else {
        // zstd::Encoder::with_dictionary(Vec::new(), level as i32, &dict_data)
        //     .expect("Failed to create zstd encoder with dictionary")
        // };

        ZstdCompressor {
            encoder: RefCell::new(Some(encoder)),
            offset: Cell::new(0),
        }
    }

    /// Receive bytes, write them into the zstd encoder, flush, and
    /// return newly produced compressed bytes (since last call).
    fn add_bytes(&self, input: Vec<u8>) -> Vec<u8> {
        let mut enc_guard = self.encoder.borrow_mut();
        let enc = enc_guard
            .as_mut()
            .expect("Compressor was already finished or uninitialized");

        enc.write_all(&input).expect("zstd write failed");
        enc.flush().expect("zstd flush failed");

        let buffer = enc.get_ref();
        let offset = self.offset.get();
        let new_chunk = buffer[offset..].to_vec();
        self.offset.set(buffer.len());

        new_chunk
    }

    fn finish(&self) -> Vec<u8> {
        let mut enc_guard = self.encoder.borrow_mut();
        let enc = enc_guard
            .take()
            .expect("Compressor was already finished or uninitialized");
        enc.finish().expect("zstd finish failed")
    }

    fn pipe_through(&self, _input: &InputStream, _output: &OutputStream) {
        todo!()
    }
}

impl Guest for BrotliCompressor {
    type Compressor = BrotliCompressor;
}

impl Guest for ZstdCompressor {
    type Compressor = ZstdCompressor;
}
bindings::export!(BrotliCompressor with_types_in bindings);
