// https://doc.rust-lang.org/cargo/reference/build-scripts.html

fn main() {
    ////////////////////////////////////////////////////////////////////////
    // ImageMagic, OSX 11.1 ARM64
    // Установка зависимости: brew install imagemagick zlib libxml2 libiconv bzip2 little-cms2
    ////////////////////////////////////////////////////////////////////////

    // TODO: Конфиг по платформам
    // Есть проблема с разными типами зависимостей libiconv, приходится добавлять стандартный маковский фреймворк
    
    // Пути поиска библиотек
    println!("cargo:rustc-link-search=/opt/homebrew/lib");
    println!("cargo:rustc-link-search=/opt/homebrew/opt/zlib/lib");
    println!("cargo:rustc-link-search=/opt/homebrew/opt/libxml2/lib");
    println!("cargo:rustc-link-search=/opt/homebrew/opt/bzip2/lib");
    println!("cargo:rustc-link-search=/opt/homebrew/opt/little-cms2/lib");
    println!("cargo:rustc-link-search=/Applications/Xcode.app/Contents/Developer/\
                Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk/usr/lib");
    // Static
    println!("cargo:rustc-link-lib=static=z");
    println!("cargo:rustc-link-lib=static=xml2");
    println!("cargo:rustc-link-lib=static=bz2");
    println!("cargo:rustc-link-lib=static=ltdl");
    println!("cargo:rustc-link-lib=static=lcms2");
    // Dynamic
    println!("cargo:rustc-link-lib=dylib=MagickCore-7.Q16HDRI");
    println!("cargo:rustc-link-lib=dylib=MagickWand-7.Q16HDRI");
    println!("cargo:rustc-link-lib=dylib=iconv");
    println!("cargo:rustc-link-lib=dylib=omp");

    ////////////////////////////////////////////////////////////////////////
    // OpenCV
    ////////////////////////////////////////////////////////////////////////

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