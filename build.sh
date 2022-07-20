cargo build
RUSTFLAGS="-C target-cpu=native" cargo build --release
cbindgen --output lua/header.lua
# read -p "..."
