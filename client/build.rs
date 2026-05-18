fn main() {
    let ca_cert_path = "../cert/ca.crt";

    println!("cargo:rerun-if-changed={ca_cert_path}");

    let ca_cert_bytes = std::fs::read(ca_cert_path).expect(
        "Failed to read CA certificate from cert/ca.crt. Please ensure certificates are generated.",
    );
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = std::path::Path::new(&out_dir).join("ca.crt");
    std::fs::write(dest_path, ca_cert_bytes).unwrap();
}
