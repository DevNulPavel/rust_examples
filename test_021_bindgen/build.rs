
// https://doc.rust-lang.org/cargo/reference/build-scripts.html
// https://doc.rust-lang.org/cargo/reference/environment-variables.html
// https://rurust.github.io/cargo-docs-ru/build-script.html

fn main() {
    // Аналог дефайнов для условной компиляции
    println!("rustc-cfg=USE_CUSTOM_LIB");

    // Флаги, передаваемые компилятору
    println!("cargo:rustc-flags=-lc++");
    println!("cargo:rustc-flags=-lstdc++");

    // Конфигурация происходит через вывод данного скрипта
    let build_type = std::env::var("PROFILE").unwrap();

    // Значение, передаваемое компилятору опцией -L, путь к библиотеке
    println!("cargo:rustc-link-search=all=libs/snappy/build_{}", build_type);    
    // Таким образом мы указываем параметры, которые передаются после -l, тип может быть static, dylib(по умолчанию), framework
    println!("cargo:rustc-link-lib=static=snappy");

    println!("cargo:rustc-link-search=all=libs/custom_lib/build_{}", build_type);
    println!("cargo:rustc-link-lib=static=custom_lib");    
}