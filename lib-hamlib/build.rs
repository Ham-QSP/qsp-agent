use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-check-cfg=cfg(hamlib_vprintf_cb_uses_va_list_pointer)");

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
    let bindings_path = out_path.join("bindings.rs");
    bindings
        .write_to_file(&bindings_path)
        .expect("Couldn't write bindings!");

    let bindings_source = fs::read_to_string(&bindings_path).expect("Couldn't read bindings");
    if let Some(vprintf_cb_pos) = bindings_source.find("pub type vprintf_cb_t") {
        let vprintf_cb_source = &bindings_source[vprintf_cb_pos..];
        let vprintf_cb_end = vprintf_cb_source
            .find("pub type freq_cb_t")
            .or_else(|| vprintf_cb_source.find("pub type chan_cb_t"))
            .unwrap_or_else(|| vprintf_cb_source.len().min(2_000));
        let vprintf_cb_section = &vprintf_cb_source[..vprintf_cb_end];
        let normalized: String = vprintf_cb_section
            .chars()
            .filter(|ch| !ch.is_whitespace())
            .collect();

        if normalized.contains("arg4:*mut__va_list_tag")
            || normalized.contains("fn(arg1:rig_debug_level_e,arg2:*mut::std::os::raw::c_void,arg3:*const::std::os::raw::c_char,arg4:*mut__va_list_tag)")
        {
            println!("cargo:rustc-cfg=hamlib_vprintf_cb_uses_va_list_pointer");
        }
    } else {
        println!("cargo:warning=Unable to locate vprintf_cb_t definition in generated bindings");
    }
}
