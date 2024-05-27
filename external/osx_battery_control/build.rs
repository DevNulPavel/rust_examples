// extern crate bindgen;

// use std::env;
// use std::path::PathBuf;

// Example custom build script.
fn main() {
    // https://doc.rust-lang.org/cargo/reference/build-scripts.html
    // println!("cargo:rerun-if-changed=smc-command/smc.c");
    println!("cargo:rustc-link-lib=framework=IOKit");
    //println!("cargo:rustc-link-lib=smc");
    // println!("cargo:rustc-link-lib=ld");

    // cc::Build::new()
    //     .file("smc-command/smc.c")
    //     // .compiler("/usr/bin/gcc")
    //     .no_default_flags(true)
    //     .flag("-mmacosx-version-min=10.4")
    //     // .flag("-framework IOKit")
    //     //.define("CMD_TOOL_BUILD", None)
    //     // .define("CMD_TOOL", None)
    //     .warnings(true)
    //     .extra_warnings(true)
    //     .include("smc-command/")
    //     .opt_level(2)
    //     .debug(true)
    //     .static_flag(true)
    //     .shared_flag(false)
    //     .compile("smc");

    //println!("cargo:rustc-link-lib=bz2");
    
    //println!("cargo:rustc-link-lib=framework=IOKit");

    // let mut headers = vec![];
    // headers.push("IOKit/IOKitLib.h");

    // let meta_header: Vec<_> = headers
    //     .iter()
    //     .map(|h| format!("#include <{}>\n", h))
    //     .collect();

    // let target = "";
    /*let target = std::env::var("TARGET").unwrap();

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        // .header("smc-command/smc.h")
        .header("test.h")
        // .header_contents("system_libs.h", &meta_header.concat())
        .clang_args(&[&format!("--target={}", target)])
        //.header("IOKit/IOKitLib.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // .clang_arg("-framework IOKit")
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");*/


    cc::Build::new()
        .file("test/test.c")
        .no_default_flags(true)
        .warnings(true)
        .extra_warnings(true)
        .opt_level(2)
        .debug(true)
        .static_flag(true)
        .shared_flag(false)
        .compile("testlib");

    cc::Build::new()
        .file("smc-command/smc.c")
        .no_default_flags(true)
        .flag("-mmacosx-version-min=10.4")
        //.define("CMD_TOOL_BUILD", None)
        //.define("CMD_TOOL", None)
        .warnings(true)
        .extra_warnings(true)
        .include("smc-command/")
        .opt_level(2)
        .debug(true)
        .static_flag(true)
        .shared_flag(false)
        .compile("smc");
}