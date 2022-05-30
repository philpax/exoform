#!/bin/bash
set -o errexit

mkdir -p build
rm -rf build/*
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen --out-name sdfbox-client --out-dir build --target web ../target/wasm32-unknown-unknown/release/sdfbox-client.wasm
cp assets/* build
rm -r ../server/assets/*
cp -r build/* ../server/assets/