#!/bin/bash

# Exit immediately if a command exits with a non-zero status.
set -e

# Function to display usage
usage() {
    echo "Usage: $0 [--debug | --release]"
    exit 1
}

# Default build mode
BUILD_MODE="debug"
CARGO_FLAGS=""

# Parse arguments
if [ "$#" -gt 1 ]; then
    usage
fi

if [ "$1" == "--release" ]; then
    BUILD_MODE="release"
    CARGO_FLAGS="--release"
elif [ "$1" == "--debug" ] || [ -z "$1" ]; then
    BUILD_MODE="debug"
    CARGO_FLAGS=""
else
    usage
fi

echo "Building in $BUILD_MODE mode..."

# Check for necessary certs and keys
CERT_DIR="cert"
REQUIRED_CERTS=("ca.crt" "schach_server.crt" "schach_server.key")

MISSING_CERTS=()
for cert in "${REQUIRED_CERTS[@]}"; do
    if [ ! -f "$CERT_DIR/$cert" ]; then
        MISSING_CERTS+=("$cert")
    fi
done

if [ "${#MISSING_CERTS[@]}" -ne 0 ]; then
    echo "Warning: The following certificate files are missing in '$CERT_DIR/': ${MISSING_CERTS[*]}"
    echo "Please create these certs first using the scripts in the '$CERT_DIR/' directory."
    echo "Build stopped."
    exit 1
fi

# Prepare client cert for include_bytes!
# The client code uses include_bytes!("../cert/ca.crt") relative to client/src/net/tls.rs
# which means it looks for client/src/cert/ca.crt
CLIENT_CERT_INTERMEDIATE_DIR="client/src/cert"
mkdir -p "$CLIENT_CERT_INTERMEDIATE_DIR"
cp "$CERT_DIR/ca.crt" "$CLIENT_CERT_INTERMEDIATE_DIR/"

# Build chess-server
echo "Building chess-server..."
cargo build -p chess-server $CARGO_FLAGS

# Build chess-client
echo "Building chess-client..."
cargo build -p chess-client $CARGO_FLAGS

# Prepare build directory
echo "Packaging into build/ directory..."
FINAL_BUILD_DIR="build"
SERVER_TARGET_DIR="$FINAL_BUILD_DIR/chess-server"
CLIENT_TARGET_DIR="$FINAL_BUILD_DIR/chess-client"

rm -rf "$FINAL_BUILD_DIR"
mkdir -p "$SERVER_TARGET_DIR"
mkdir -p "$CLIENT_TARGET_DIR"

# Copy binaries
# Binaries are located in target/debug or target/release
if [ "$BUILD_MODE" == "release" ]; then
    cp "target/release/chess-server" "$SERVER_TARGET_DIR/"
    cp "target/release/chess-client" "$CLIENT_TARGET_DIR/"
else
    cp "target/debug/chess-server" "$SERVER_TARGET_DIR/"
    cp "target/debug/chess-client" "$CLIENT_TARGET_DIR/"
fi

# Copy certs for server
cp -r "$CERT_DIR" "$SERVER_TARGET_DIR/"

# Copy client assets
CLIENT_ASSET_DIR="client/assets/"
cp -r "$CLIENT_ASSET_DIR" "$CLIENT_TARGET_DIR"
mv "$CLIENT_TARGET_DIR/assets/default_settings.cfg" "$CLIENT_TARGET_DIR/settings.cfg"

# Cleanup intermediate cert
echo "Cleaning up..."
rm -rf "$CLIENT_CERT_INTERMEDIATE_DIR"

echo "Build successful! Artifacts are in $FINAL_BUILD_DIR/"
