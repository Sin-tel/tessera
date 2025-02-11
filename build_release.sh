#!/bin/bash
# Build the release version of the project

export RELEASE=true
cargo build --release
