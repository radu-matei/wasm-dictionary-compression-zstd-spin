#[allow(warnings)]
mod bindings;

use bindings::exports::component::compressor::compress::{Guest, GuestCompressor};

use std::{
    cell::{Cell, RefCell},
    io::Write,
};

use brotli::CompressorWriter;

pub struct MyCompressor {
    writer: RefCell<Option<CompressorWriter<Vec<u8>>>>,
    offset: Cell<usize>,
}

impl GuestCompressor for MyCompressor {
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
}

impl Guest for MyCompressor {
    type Compressor = MyCompressor;
}

bindings::export!(MyCompressor with_types_in bindings);
