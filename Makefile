
.PHONY: compressor
compressor:
	cd compressor && RUSTFLAGS=-Ctarget-feature=+simd128 cargo component build --release

.PHONY: dictionaries
dictionaries:
	zstd --train assets/* -o dictionaries/v1.dict --maxdict=65536	

