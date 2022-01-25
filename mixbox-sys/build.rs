#![allow(unreachable_code)]
use std::{env, path::PathBuf};

//#[cfg(all(target_os = "macos", feature = "openmp"))]
//static MAC_OS_BREW_CLANG_PATH: &str = "/usr/local/opt/llvm";

pub fn main() {

    cc::Build::new()
        //.include(&osd_inlude_path)
        .cpp(true)
        .static_flag(true)
        .flag("-std=c++11")
        .flag("-Wno-sign-compare")
        .flag("-Wno-unused-function")
        .flag("-Wno-unused-parameter")
        .file("mixbox/mixbox.cpp")

        .compile("mixbox");

    println!("cargo:rustc-link-lib=static=mixbox");

    println!("cargo:rerun-if-changed=wrapper.hpp");

    let bindings = bindgen::Builder::default()
        .header("wrapper.hpp")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        /*.allowlist_type("OpenSubdiv.*")
        .derive_partialeq(true)
        .derive_eq(true)
        .derive_hash(true)
        .derive_debug(true)*/
        .layout_tests(false);

    let out_path = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let bindings = bindings
        .clang_args(&["-F", "mixbox"])
        .generate()
        .expect("Unable to generate bindings");

    let bindings_path = out_path.join("bindings.rs");
    bindings
        .write_to_file(&bindings_path)
        .expect("Couldn't write bindings");

    println!("cargo:rerun-if-changed=build.rs");
}
