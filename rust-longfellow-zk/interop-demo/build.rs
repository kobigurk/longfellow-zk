use std::env;
use std::path::PathBuf;

fn main() {
    // Generate C header for FFI bindings
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    
    // Configure cbindgen to generate C headers
    let config = cbindgen::Config {
        language: cbindgen::Language::C,
        cpp_compat: true,
        include_guard: Some("LONGFELLOW_FFI_H".to_string()),
        namespace: Some("longfellow".to_string()),
        export: cbindgen::ExportConfig {
            include: vec!["longfellow_.*".to_string()],
            ..Default::default()
        },
        ..Default::default()
    };
    
    // Generate the header
    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_config(config)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_path.join("longfellow_ffi.h"));
    
    // Print cargo directives
    println!("cargo:rerun-if-changed=src/cpp_verifier_interop.rs");
    println!("cargo:rerun-if-changed=build.rs");
}