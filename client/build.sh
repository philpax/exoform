#!/bin/sh
cargo build --release && wasm-bindgen --out-name sdfbox-client --out-dir target --target web target/wasm32-unknown-unknown/release/sdfbox-client.wasm