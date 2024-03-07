# Simple physics engine written in Rust

## About
This is my attempt at writing simple physics engine using Rust. For visualization, I'm using WGPU library. 

# Usage
To run it locally use:
```shell
cargo run
```
To run it in web browser using wasm server run:
```shell
WASM_SERVER_RUNNER_CUSTOM_INDEX_HTML="./index.html" cargo watch -x "run --target wasm32-unknown-unknown"
```
Setting WASM_SERVER_RUNNER_CUSTOM_INDEX_HTML might not be necessary but for me, it wasn't properly detecting html file.
