# Zstandard Dictionary Compression in Spin

This is an example that shows dictionary compression using Zstandard using Spin and Wasm components.

It consists of the following components:

* a Wasm component that performs zstd dictionary compression
* a TypeScript API that uses the compression component 
* a static file server

This uses the comopnent composition feature in Spin to link the compression Wasm component written in Rust, with the
TypeScript API.


### Building and running

Prerequisites:

* [Make](https://www.gnu.org/software/make/) — used for executing the make target
* [WASI SDK](https://github.com/WebAssembly/wasi-sdk), [Rust](https://www.rust-lang.org/tools/install), and [`cargo component`](https://github.com/bytecodealliance/cargo-component)  — used for compiling the Wasm component used for compression
* [Node.js](https://nodejs.org/en) and [NPM](https://www.npmjs.com/) — used for building the TypeScript API into a Wasm component
* [Spin](https://github.com/fermyon/spin) — used for running the overall function

```
# set the WASI_SDK variable to your installation directory
$ make
$ spin up
Logging component stdio to ".spin/logs/"

Serving http://127.0.0.1:3000
Available Routes:
  assets: http://127.0.0.1:3000/static/assets (wildcard)
  api: http://127.0.0.1:3000 (wildcard)
```

To verify the function works as expected, you can make requests using curl:

```
# compressed
$ time curl localhost:3000/stream/small.txt -H "Available-Dictionary: v1.dict" -H "Accept-Encoding: dcz" -H "Dictionary-Id: v1.dict" --output compressed-js.zst
  % Total    % Received % Xferd  Average Speed   Time    Time     Time  Current
                                 Dload  Upload   Total   Spent    Left  Speed
100   269    0   269    0     0  34412      0 --:--:-- --:--:-- --:--:-- 38428
curl localhost:3000/stream/small.txt -H "Available-Dictionary: v1.dict" -H  -  0.01s user 0.01s system 50% cpu 0.028 total


# uncompressed
$ time curl localhost:3000/stream/small.txt --output uncompressed-js.txt
  % Total    % Received % Xferd  Average Speed   Time    Time     Time  Current
                                 Dload  Upload   Total   Spent    Left  Speed
100 17208    0 17208    0     0  2187k      0 --:--:-- --:--:-- --:--:-- 2400k
curl localhost:3000/stream/small.txt --output uncompressed-js.txt  0.01s user 0.01s system 55% cpu 0.031 total
```

You can validate the compression was correct by uncompressing and checking against the original file:

```
$ zstd -D assets/dictionaries/v1.dict -d compressed-js.zst -o uncompressed.txt
compressed-js.zst   : 17208 bytes

$ sha256sum assets/train/small.txt uncompressed.txt
5a04a432dc175205f453355d956dcc0d239be1168c8f20a3b7e87a282fc6b115  assets/train/small.txt
5a04a432dc175205f453355d956dcc0d239be1168c8f20a3b7e87a282fc6b115  uncompressed.txt
```

The compression behavior is controlled by the headers according to the [IETF draft for dictionary compression](https://datatracker.ietf.org/doc/html/draft-ietf-httpbis-compression-dictionary-19).

> Note: this project relies on an [unreleased version of ComponentizeJS based on the bug fix](https://github.com/bytecodealliance/ComponentizeJS/pull/184). Once a new release is out, it will be updated.
