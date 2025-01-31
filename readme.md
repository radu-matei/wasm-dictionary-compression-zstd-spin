# Dictionary compression in Spin

This example that intends to show dictionary compression for streaming responses in Spin.
This repository is currently an internal bug repro, not a fully working sample.

For files larger than a few KiB, compression/streaming silently fails.
Note: originally this was using Zstd for compression, but the same setup did not result in valid compressed files even for small files.


### Building

```
$ make compressor
$ spin build && spin up
```

You can now make requests:

* for a 17 KiB file:

```
❯ curl localhost:3000/stream/small.txt -H "Available-Dictionary: abc" -H "Dictionary-Id: v1.dict" --output compressed.br
  % Total    % Received % Xferd  Average Speed   Time    Time     Time  Current
                                 Dload  Upload   Total   Spent    Left  Speed
100   301    0   301    0     0  10944      0 --:--:-- --:--:-- --:--:-- 11148

❯ brotli -d compressed.br -o small.txt

❯ sha256sum small.txt
5a04a432dc175205f453355d956dcc0d239be1168c8f20a3b7e87a282fc6b115  small.txt

❯ sha256sum assets/small.txt
5a04a432dc175205f453355d956dcc0d239be1168c8f20a3b7e87a282fc6b115  assets/small.txt
```

* for a 6 MiB file:

```
❯ curl localhost:3000/stream/big.txt -H "Available-Dictionary: abc" -H "Dictionary-Id: v1.dict" --output compressed.br
  % Total    % Received % Xferd  Average Speed   Time    Time     Time  Current
                                 Dload  Upload   Total   Spent    Left  Speed
100  308k    0  308k    0     0   365k      0 --:--:-- --:--:-- --:--:--  365k

❯ ls compressed.br
316k 26 seconds  compressed.br

❯ brotli -d compressed.br -o big.txt
corrupt input [compressed.br]
```

This generates the following error in Spin (only visible when running with RUST_LOG=spin=warn at least) enabled:

```
2025-01-31T02:08:20.619207Z  WARN spin_trigger_http::server: Error serving HTTP connection: hyper::Error(Io, Os { code: 54, kind: ConnectionReset, message: "Connection reset by peer" })
Error sending body: Error::ProtocolError("I/O error: StreamError::Closed")
2025-01-31T02:08:20.619297Z  WARN spin_trigger_http::wasi: component error after response: guest invocation failed

Caused by:
    0: error while executing at wasm backtrace:
           0: 0x6b1401 - <unknown>!<wasm function 10148>
           1: 0x3505a6 - <unknown>!<wasm function 5142>
           2: 0x37e506 - <unknown>!<wasm function 5407>
           3: 0x37f139 - <unknown>!<wasm function 5410>
           4: 0x37f957 - <unknown>!<wasm function 5413>
           5: 0x484161 - <unknown>!<wasm function 7221>
           6: 0x3830f9 - <unknown>!<wasm function 5441>
           7: 0x375db3 - <unknown>!<wasm function 5406>
           8: 0x36fd09 - <unknown>!<wasm function 5404>
           9: 0x37e3cc - <unknown>!<wasm function 5407>
          10: 0x37f139 - <unknown>!<wasm function 5410>
          11: 0x3ffb54 - <unknown>!<wasm function 6403>
          12: 0x1747fa - <unknown>!<wasm function 588>
          13: 0x174a7f - <unknown>!<wasm function 590>
          14: 0x1732f0 - <unknown>!<wasm function 574>
          15: 0x37e506 - <unknown>!<wasm function 5407>
          16: 0x37f139 - <unknown>!<wasm function 5410>
          17: 0x46c787 - <unknown>!<wasm function 7127>
          18: 0x37e506 - <unknown>!<wasm function 5407>
          19: 0x37f139 - <unknown>!<wasm function 5410>
          20: 0x3ffb54 - <unknown>!<wasm function 6403>
          21: 0x43c6c6 - <unknown>!<wasm function 6801>
          22: 0x13f629 - <unknown>!<wasm function 168>
          23: 0x18b532 - <unknown>!<wasm function 718>
          24: 0x144c84 - <unknown>!<wasm function 263>
    1: wasm trap: wasm `unreachable` instruction executed
```

Another implementation built using Zstd failed in similar ways, but without the hyper error. So not sure it's actually a networking error. 



# Update

A similar consumer component written in Rust appears to work:

```
❯ curl localhost:3000/rustapi/stream/big.txt -H "Available-Dictionary: abc" --output compressed.br
  % Total    % Received % Xferd  Average Speed   Time    Time     Time  Current
                                 Dload  Upload   Total   Spent    Left  Speed
100 1890k    0 1890k    0     0   303k      0 --:--:--  0:00:06 --:--:--  293k

❯ brotli -d compressed.br -o big.txt

❯ sha256sum big.txt
fa066c7d40f0f201ac4144e652aa62430e58a6b3805ec70650f678da5804e87b  big.txt

❯ sha256sum assets/big.txt
fa066c7d40f0f201ac4144e652aa62430e58a6b3805ec70650f678da5804e87b  assets/big.txt
```
