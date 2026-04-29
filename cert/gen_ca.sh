#!/usr/bin/env bash

# Exit immediately if a command exits with a non-zero status
set -euo pipefail

KEY_FILE="ca.key"
CRT_FILE="ca.crt"

echo "=== CA Certificate Generation ==="

# Check if the private key exists
if [ -f "$KEY_FILE" ]; then
    echo "-> [INFO] '$KEY_FILE' found. Using existing key."
else
    echo "========================================================================"
    echo "Error: Required file '$KEY_FILE' is missing in this directory."
    echo "========================================================================"
    echo "Please provide your own private key as '$KEY_FILE', inside the directory of this script"
    echo "or generate a new one by running the following command:"
    echo ""
    echo "    openssl genrsa -out ca.key 4096"
    echo ""
    echo "Exiting."
    exit 1
fi

# Generate the CA certificate 
echo "-> Generating CA certificate..."
openssl req -x509 -new -nodes \
    -key "$KEY_FILE" \
    -sha256 \
    -days 3650 \
    -out "$CRT_FILE" \
    -subj "/CN=Schach! CA"

echo "=== Done! ==="
echo "CA Key:         $KEY_FILE"
echo "CA Certificate: $CRT_FILE"

