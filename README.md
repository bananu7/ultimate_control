# ultimate_control
REaRiiR (Reverse Engineering and Rewriting it in Rust) of Presonus Universal Control software

Looking for a way to control StudioLive mixers via UCNET? See if [this repo](https://github.com/featherbear/presonus-studiolive-api) can help.

## Building

Requires Rust 1.85 or newer (edition 2024). Install via [rustup](https://rustup.rs), then:

```sh
cargo build
cargo test
```

## Usage

Universal Control must be running: it hosts the UCNET server on
`127.0.0.1:49670` that the device is reached through.

```sh
cargo run --bin app -- monitor              # print all traffic, reconnects on drop
cargo run --bin app -- dump-state           # every parameter path and value
cargo run --bin app -- get main/ch1/mute
cargo run --bin app -- hardware-mute on     # what the physical mute button does
cargo run --bin app -- main-mute on         # software main-bus mute UC doesn't expose
cargo run --bin app -- set line/ch1/volume 0.735
```

`set` and the mute commands wait for the device to echo the new value back,
so a confirmation line means the device accepted it. Known parameter paths
are in [`src/params.rs`](src/params.rs); `dump-state` lists everything your
device actually exposes.

Protocol notes live in [`doc/`](doc/).
