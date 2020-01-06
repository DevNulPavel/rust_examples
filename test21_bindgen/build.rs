
// https://doc.rust-lang.org/cargo/reference/build-scripts.html
// https://doc.rust-lang.org/cargo/reference/environment-variables.html

fn main() {
    // Конфигурация происходит через вывод данного скрипта
    let build_type = std::env::var("PROFILE").unwrap();
    println!("{}", format!("cargo:rustc-flags=-L libs/snappy/build_{}", build_type));
    println!("{}", format!("cargo:rustc-link-search=all=libs/snappy/build_{}", build_type));    
    println!("cargo:rustc-link-lib=static=snappy");
}