// https://doc.rust-lang.org/cargo/reference/build-scripts.html
use std::{
    path::{
        PathBuf
    },
    env::{
        self
    }
};


fn main() {
    // TODO: Конфиг по платформам
    // brew install imagemagick
    
    println!("cargo:rustc-link-search=/opt/homebrew/lib");
    println!("cargo:rustc-link-lib=static=MagickCore-7.Q16HDRI");
    println!("cargo:rustc-link-lib=static=MagickWand-7.Q16HDRI");

    /*println!("cargo:rustc-link-lib=framework=AppKit");
    println!("cargo:rustc-link-lib=framework=OpenCL");
    println!("cargo:rustc-link-lib=dylib=c++");
    println!("cargo:rustc-link-lib=static=ittnotify");
    println!("cargo:rustc-link-lib=static=tbb");
    println!("cargo:rustc-link-lib=static=opencv_core");
    println!("cargo:rustc-link-lib=static=opencv_imgproc");
    println!("cargo:rustc-link-lib=static=opencv_imgcodecs");*/

    // Result: target/debug/build/test56_rest-ee0bebc12d7aaccf/out/bindings.rs
    /*println!("cargo:rerun-if-changed=wrapper.h");
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");*/
}