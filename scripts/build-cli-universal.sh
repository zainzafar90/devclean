#!/bin/bash
# Builds the devclean CLI for Apple Silicon and Intel and joins them into one universal binary.
set -euo pipefail
cd "$(dirname "$0")/.."
targets="aarch64-apple-darwin x86_64-apple-darwin"
if command -v rustup >/dev/null; then rustup target add $targets >/dev/null 2>&1; fi
for triple in $targets; do cargo build --quiet --release -p devclean-cli --target "$triple"; done
out="target/cli-universal/devclean"
mkdir -p "$(dirname "$out")"
lipo -create -output "$out" target/aarch64-apple-darwin/release/devclean target/x86_64-apple-darwin/release/devclean
lipo -info "$out"
