#!/bin/bash
set -e

echo "Starting build script..."
echo "Current directory: $(pwd)"
ls -la

# Check if rustup is installed
if ! command -v rustup &> /dev/null; then
    echo "Rustup not found. Installing..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    echo "Rustup already installed."
fi

# Ensure cargo is in PATH (in case it wasn't added by source above)
export PATH="$HOME/.cargo/bin:$PATH"

# Install Rust target
echo "Installing wasm32-unknown-unknown..."
rustup target add wasm32-unknown-unknown

# Install Trunk
# We use cargo install instead of downloading a binary because Vercel's Amazon Linux 2 environment
# has an older GLIBC that is incompatible with the standard pre-compiled trunk binaries.
if ! command -v trunk &> /dev/null; then
    echo "Trunk not found. Installing via cargo (this may take a few minutes)..."
    cargo install trunk
else
    echo "Trunk already installed."
fi

echo "Trunk version: $(trunk --version)"

echo "Building with Trunk..."
trunk build --release
