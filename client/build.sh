#!/bin/bash
set -o errexit

mkdir -p build
rm -rf build/*
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen --out-name exoform-client --out-dir build --target web ../target/wasm32-unknown-unknown/release/exoform-client.wasm
cp assets/* build
mkdir -p ../server/assets
rm -rf ../server/assets/*
cp -r build/* ../server/assets/