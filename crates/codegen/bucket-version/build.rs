fn main() {
    println!("cargo:rerun-if-env-changed=BUCKET_VERSION");
}
