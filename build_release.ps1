#!/usr/bin/env pwsh

# Set environment variable for release build
$env:RELEASE = "true"

# Run cargo build and capture exit code
cargo build --release
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}
