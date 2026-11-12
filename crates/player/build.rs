fn main() {
    if cfg!(windows) {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../third_party/mpv");
        println!("cargo:rustc-link-search=native={}", dir.display());
    }
    println!("cargo:rerun-if-changed=build.rs");
}
