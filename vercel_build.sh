#!/bin/bash
set -e

echo "Starting build script..."
echo "Current directory: $(pwd)"
ls -la

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
