#!/bin/bash
set -e

# Install Rust target
rustup target add wasm32-unknown-unknown

# Download Trunk binary (faster than cargo install)
TRUNK_VERSION="v0.21.5"
if ! command -v trunk &> /dev/null; then
    echo "Downloading Trunk $TRUNK_VERSION..."
    wget -qO- https://github.com/trunk-rs/trunk/releases/download/$TRUNK_VERSION/trunk-x86_64-unknown-linux-gnu.tar.gz | tar -xzf- -C .
    chmod +x trunk
    # Add current directory to PATH so we can use it
    export PATH=$PATH:$(pwd)
fi

echo "Building with Trunk..."
# Ensure dependencies are downloaded (Cargo will handle this)
trunk build --release
