@echo off
set RELEASE=true
cargo build --release || exit /b 1
