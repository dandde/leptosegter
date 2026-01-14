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

# Download Trunk binary
TRUNK_VERSION="v0.21.5"
if ! command -v trunk &> /dev/null; then
    echo "Trunk not found, downloading $TRUNK_VERSION..."
    if command -v wget &> /dev/null; then
        echo "Using wget..."
        wget -qO- https://github.com/trunk-rs/trunk/releases/download/$TRUNK_VERSION/trunk-x86_64-unknown-linux-gnu.tar.gz | tar -xzf- -C .
    else
        echo "Using curl..."
        curl -L https://github.com/trunk-rs/trunk/releases/download/$TRUNK_VERSION/trunk-x86_64-unknown-linux-gnu.tar.gz | tar -xzf- -C .
    fi
    chmod +x trunk
    export PATH=$PATH:$(pwd)
else
    echo "Trunk already installed."
fi

echo "Trunk version: $(trunk --version)"

echo "Building with Trunk..."
trunk build --release
