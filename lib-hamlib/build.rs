use std::env;
use std::path::PathBuf;

fn main() {
    let mut builder = bindgen::Builder::default().header("wrapper.h");

    if let Ok(library) = pkg_config::Config::new().probe("hamlib") {
        for include_path in library.include_paths {
            builder = builder.clang_arg(format!("-I{}", include_path.display()));
        }
    } else if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-search=/opt/homebrew/lib");
        println!("cargo:rustc-link-lib=hamlib");
        builder = builder.clang_arg("-I/opt/homebrew/include");
    } else {
        println!("cargo:rustc-link-lib=hamlib");
    }

    let bindings = builder
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
