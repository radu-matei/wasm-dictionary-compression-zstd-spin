WASI_SDK_PATH := /opt/wasi-sdk
CC := $(WASI_SDK_PATH)/bin/clang --sysroot=$(WASI_SDK_PATH)/share/wasi-sysroot


build: compressor test spin
test: 

.PHONY: spin
spin:
	spin build

.PHONY: compressor
compressor:
	cd compressor && \
		WASI_SDK_PATH="$(WASI_SDK_PATH)" \
		CC="$(CC)" \
		RUSTFLAGS=-Ctarget-feature=+simd128 \
		cargo +nightly component build --release

.PHONY: test
test:
	cd compressor && \
		WASI_SDK_PATH="$(WASI_SDK_PATH)" \
		CC="$(CC)" \
		RUSTFLAGS=-Ctarget-feature=+simd128 \
		CARGO_TARGET_WASM32_WASIP1_RUNNER="wasmtime --dir ." cargo +nightly component test -- --nocapture

.PHONY: dictionaries
dictionaries:
	zstd --train assets/train/* -o assets/dictionaries/v1.dict --maxdict=65536	

