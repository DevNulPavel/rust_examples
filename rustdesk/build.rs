#[cfg(windows)]
fn build_windows() {
    cc::Build::new().file("src/windows.cc").compile("windows");
    println!("cargo:rustc-link-lib=WtsApi32");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=windows.cc");
}

#[cfg(all(windows, feature = "inline"))]
fn build_manifest() {
    use std::io::Write;
    if std::env::var("PROFILE").unwrap() == "release" {
        let mut res = winres::WindowsResource::new();
        res.set_icon("icon.ico")
            .set_language(winapi::um::winnt::MAKELANGID(
                winapi::um::winnt::LANG_ENGLISH,
                winapi::um::winnt::SUBLANG_ENGLISH_US,
            ))
            .set_manifest_file("manifest.xml");
        match res.compile() {
            Err(e) => {
                write!(std::io::stderr(), "{}", e).unwrap();
                std::process::exit(1);
            }
            Ok(_) => {}
        }
    }
}

fn install_oboe() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    if target_os != "android" {
        return;
    }
    let mut target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    if target_arch == "x86_64" {
        target_arch = "x64".to_owned();
    } else if target_arch == "aarch64" {
        target_arch = "arm64".to_owned();
    } else {
        target_arch = "arm".to_owned();
    }
    let target = format!("{}-android-static", target_arch);
    let vcpkg_root = std::env::var("VCPKG_ROOT").unwrap();
    let mut path: std::path::PathBuf = vcpkg_root.into();
    path.push("installed");
    path.push(target);
    println!(
        "{}",
        format!(
            "cargo:rustc-link-search={}",
            path.join("lib").to_str().unwrap()
        )
    );
    println!("cargo:rustc-link-lib=oboe");
    println!("cargo:rustc-link-lib=c++");
    println!("cargo:rustc-link-lib=OpenSLES");
    // I always got some strange link error with oboe, so as workaround, put oboe.cc into oboe src: src/common/AudioStreamBuilder.cpp
    // also to avoid libc++_shared not found issue, cp ndk's libc++_shared.so to jniLibs, e.g.
    // ./flutter_hbb/android/app/src/main/jniLibs/arm64-v8a/libc++_shared.so
    // let include = path.join("include");
    //cc::Build::new().file("oboe.cc").include(include).compile("oboe_wrapper");
}

fn main() {
    #[cfg(all(windows, feature = "inline"))]
    build_manifest();
    #[cfg(windows)]
    build_windows();
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-lib=framework=ApplicationServices");
    hbb_common::gen_version();
    install_oboe();
}
