fn main() {
    let ca_cert_path = "../cert/ca.crt";

    // Tell Cargo to rerun this script if the certificate changes
    println!("cargo:rerun-if-changed={}", ca_cert_path);

    // Read the certificate bytes
    let ca_cert_bytes = std::fs::read(ca_cert_path).expect(
        "Failed to read CA certificate from cert/ca.crt. Please ensure certificates are generated.",
    );

    // We can write it to OUT_DIR and include it from there
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = std::path::Path::new(&out_dir).join("ca.crt");
    std::fs::write(&dest_path, ca_cert_bytes).unwrap();
}
