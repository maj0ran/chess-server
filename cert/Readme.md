## This folder contains scripts to generate certificates and keys for TLS encryption of our chess application.

TLS encryption is needed for user logins who will use their own passwords. We don't want them to expose their passwords in plain text.


### Here is the conceptual view of the certificate system:

We are our own Certificate Authority (CA). This means, we create a root certificate with a private key, named `ca.crt` and `ca.key`. 

`ca.key` can be any RSA key. If you don't have one, generate it with:

$ openssl genrsa -out ca.key 4096

The script `gen_ca.sh` generates the root certificate `ca.crt` using `ca.key`. `ca.crt` will be shipped with the chess-client during and thus deployed to each user.

The script `gen_server_cert.sh` will use `ca.crt` and `ca.key` to generate `schach_server.crt` and `schach_server.key`. The chess-server will read those files during startup and setup its TLS encrption with them.

Because any chess-client has `ca.crt` with it, it will accept connections with any server that has `schach_server.crt` and `schach_server.key` generated from the same `ca.crt`. 

**The server certificate will expire after 365 days.** However, we can generate a new server cert with the same `ca.crt` as before and all clients will still accept the server.

**The CA certificate will expire after 3650 days.** Then we will have to generate a new CA cert, re-generate all server certificates and re-compile all clients with the new `ca.crt`.

