

fn main() {
    // Создает cc::Build объект для дальнейшей настройки сборки
    cxx_build::bridge("src/main.rs")
        .file("libs/cpp_test_lib/src/blobstore.cc")
        .flag_if_supported("-std=c++17")
        .opt_level(2)
        .compile("test76_cxx_ffi");

    println!("cargo:rerun-if-changed=src/main.rs");
    println!("cargo:rerun-if-changed=libs/cpp_test_lib/src/blobstore.cc");
    println!("cargo:rerun-if-changed=libs/cpp_test_lib/include/blobstore.h");
}