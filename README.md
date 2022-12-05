# cw-bidding-platform

```sh
cargo build

cargo test

cargo wasm
cosmwasm-check ./target/wasm32-unknown-unknown/release/cw_bidding_platform.wasm

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer-arm64:0.12.8
```
