# ultimate_control
REaRiiR (Reverse Engineering and Rewriting it in Rust) of Presonus Universal Control software

Looking for a way to control StudioLive mixers via UCNET? See if [this repo](https://github.com/featherbear/presonus-studiolive-api) can help.

## Building

Requires Rust 1.85 or newer (edition 2024). Install via [rustup](https://rustup.rs), then:

```sh
cargo build
cargo test
cargo run --bin app   # expects a UC server on 127.0.0.1:49670
```

Protocol notes live in [`doc/`](doc/).
