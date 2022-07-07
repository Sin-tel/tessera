RUSTFLAGS="-C target-cpu=native" cargo build --release
cargo build
cbindgen --output lua/header.lua