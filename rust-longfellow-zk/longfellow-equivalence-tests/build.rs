use std::env;
use std::path::PathBuf;

fn main() {
    // Compile C++ test wrapper
    cc::Build::new()
        .cpp(true)
        .file("cpp_tests/test_wrapper.cc")
        .include("../../lib")
        .include("cpp_tests")
        .flag("-std=c++17")
        .flag("-O3")
        .compile("cpp_test_wrapper");

    // Generate bindings
    let bindings = bindgen::Builder::default()
        .header("cpp_tests/test_wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // Link to the C++ library
    println!("cargo:rustc-link-search=../../clang-build-release");
    println!("cargo:rustc-link-lib=longfellow");
    
    // Rerun if files change
    println!("cargo:rerun-if-changed=cpp_tests/test_wrapper.h");
    println!("cargo:rerun-if-changed=cpp_tests/test_wrapper.cc");
}