#!/bin/bash
set -e

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check if trunk is installed
if ! command_exists trunk; then
    echo "Trunk is not installed."
    echo "Please install it with: cargo install trunk"
    exit 1
fi

echo "Building and serving the application..."
echo "Opening in default browser..."

# Serve the app and open the default browser
# --open tells trunk to open the default browser
trunk serve --open 
