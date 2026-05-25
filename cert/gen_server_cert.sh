#!/usr/bin/env bash

# Exit immediately if a command exits with a non-zero status
set -euo pipefail

if [ "$#" -lt 3 ] || [ "$#" -gt 4 ]; then
    echo "Error: Missing arguments."
    echo "Usage: $0 <CA_CERT_PATH> <CA_KEY_PATH> <HOSTNAME> [<IP_ADDRESS>]"
    exit 1
fi

CA_CERT="$1"
CA_KEY="$2"
HOSTNAME="$3"
IP_ADDRESS="${4:-}"

# Verify CA files exist
if [ ! -f "$CA_CERT" ] || [ ! -f "$CA_KEY" ]; then
    echo "Error: CA certificate or CA key file not found."
    exit 1
fi

SERVER_KEY="schach_server.key"
SERVER_CRT="schach_server.crt"
SERVER_CSR="schach_server.csr"
EXT_FILE="schach_server.ext"

echo "-> Generating CA and Key for ${HOSTNAME} (${IP_ADDRESS})."

# Generate Server Private Key
openssl genrsa -out "$SERVER_KEY" 4096

# Generate Certificate Signing Request (CSR)
openssl req -new \
    -key "$SERVER_KEY" \
    -out "$SERVER_CSR" \
    -subj "/C=DE/ST=Niedersachsen/L=Hildesheim/CN=${HOSTNAME}"

# Create a temporary SAN extension file
cat <<EOF > "$EXT_FILE"
authorityKeyIdentifier=keyid,issuer
basicConstraints=CA:FALSE
keyUsage = digitalSignature, nonRepudiation, keyEncipherment, dataEncipherment
subjectAltName = @alt_names

[alt_names]
DNS.1 = ${HOSTNAME}
DNS.2 = localhost
${IP_ADDRESS:+IP.1 = ${IP_ADDRESS}}
IP.2 = 127.0.0.1
EOF

# Sign the CSR using the CA certificate and private key
openssl x509 -req \
    -in "$SERVER_CSR" \
    -CA "$CA_CERT" \
    -CAkey "$CA_KEY" \
    -CAcreateserial \
    -out "$SERVER_CRT" \
    -days 365 \
    -sha256 \
    -extfile "$EXT_FILE"

# Clean up temporary files
rm -f "$SERVER_CSR" "$EXT_FILE"

echo "=== Done! ==="
echo "Private Key: $SERVER_KEY"
echo "Certificate: $SERVER_CRT"

