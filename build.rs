

fn main() {
    // Tell cargo to re-run this build script if any of these files change
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/port_forwarding_client.rs");
    println!("cargo:rerun-if-changed=src/port_forwarding_server.rs");
    println!("cargo:rerun-if-changed=src/config.rs");
    
    // Generate C bindings if the "ffi" feature is enabled
    #[cfg(feature = "ffi")]
    generate_bindings();
}

#[cfg(feature = "ffi")]
fn generate_bindings() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let package_name = env::var("CARGO_PKG_NAME").unwrap();
    let output_file = PathBuf::from(&crate_dir)
        .join("include")
        .join(format!("{}.h", package_name));
    
    // Create include directory if it doesn't exist
    std::fs::create_dir_all(output_file.parent().unwrap()).unwrap();
    
    // Configure and generate bindings
    let config = cbindgen::Config::from_root_or_default(&crate_dir);
    
    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_config(config)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(&output_file);
        
    println!("cargo:rerun-if-changed=include");
    println!("cargo:rustc-link-lib=static={}", package_name);
}
