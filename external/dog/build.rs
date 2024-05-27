//! Данный скрипт запускается в процессе каждого билда
//! Его цель собрать вместе файлики, используемые для --help и --version,
//! которые нужны для цветного и не цветного вариантов.
//! Главный текст содержится в src/usage.txt.
//! Для упрощения редактирования, обратные слеши используются вместо исходных
//! ANSI кодов.
//! 
//! Строка версии достаточно слоная, мы хотим показать версию, текущий хеш и дату компиляции 
//! для debug сборки.
//! И лишь версию для релизной сборки.
//! Данный скрипт генерирует строку из переменной окружения, которые добавляет Cargo
//! https://doc.crates.io/environment-variables.html
//! и запускает git для получения SHA1 хеша.
//! Затем записывает строки в файлики, которые мы можем включать во время компиляции в код.

use std::env;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;
use datetime::{LocalDateTime, ISO};


/// The build script entry point.
fn main() -> io::Result<()> {
    #![allow(clippy::write_with_newline)]

    // Строка с использованием
    let usage   = include_str!("src/usage.txt");
    // Описание с цветом
    let tagline = "dog \\1;32m●\\0m command-line DNS client";
    // Сайт
    let url     = "https://dns.lookup.dog/";

    // Строка версии
    let ver =
        if is_debug_build() {
            format!("{}\nv{} \\1;31m(pre-release debug build!)\\0m\n\\1;4;34m{}\\0m", tagline, version_string(), url)
        }
        else if is_development_version() {
            format!("{}\nv{} [{}] built on {} \\1;31m(pre-release!)\\0m\n\\1;4;34m{}\\0m", tagline, version_string(), git_hash(), build_date(), url)
        }
        else {
            format!("{}\nv{}\n\\1;4;34m{}\\0m", tagline, version_string(), url)
        };

    // We need to create these files in the Cargo output directory.
    // Файлики складываем в директорию выходных данных
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Текст с версией
    let mut f = File::create(&out.join("version.pretty.txt"))?;
    writeln!(f, "{}", convert_codes(&ver))?;

    // Красивый текст версии
    let mut f = File::create(&out.join("version.bland.txt"))?;
    writeln!(f, "{}", strip_codes(&ver))?;

    // Описание использования
    let mut f = File::create(&out.join("usage.pretty.txt"))?;
    writeln!(f, "{}", convert_codes(tagline))?;
    writeln!(f)?;
    write!(f, "{}", convert_codes(usage))?;

    // Bland usage text
    let mut f = File::create(&out.join("usage.bland.txt"))?;
    writeln!(f, "{}", strip_codes(tagline))?;
    writeln!(f)?;
    write!(f, "{}", strip_codes(usage))?;

    Ok(())
}

/// Конвертация кодов в ANSI
fn convert_codes(input: &str) -> String {
    input.replace('\\', "\x1B[")
}

/// Removes escape codes from ‘usage.txt’.
fn strip_codes(input: &str) -> String {
    input.replace("\\0m", "")
         .replace("\\1m", "")
         .replace("\\4m", "")
         .replace("\\32m", "")
         .replace("\\33m", "")
         .replace("\\1;31m", "")
         .replace("\\1;32m", "")
         .replace("\\1;33m", "")
         .replace("\\1;4;34", "")
}

/// Retrieve the project’s current Git hash, as a string.
fn git_hash() -> String {
    use std::process::Command;

    String::from_utf8_lossy(
        &Command::new("git")
            .args(&["rev-parse", "--short", "HEAD"])
            .output().unwrap()
            .stdout).trim().to_string()
}

/// Whether we should show pre-release info in the version string.
///
/// Both weekly releases and actual releases are --release releases,
/// but actual releases will have a proper version number.
fn is_development_version() -> bool {
    cargo_version().ends_with("-pre") || env::var("PROFILE").unwrap() == "debug"
}

/// Whether we are building in debug mode.
fn is_debug_build() -> bool {
    env::var("PROFILE").unwrap() == "debug"
}

/// Retrieves the [package] version in Cargo.toml as a string.
fn cargo_version() -> String {
    env::var("CARGO_PKG_VERSION").unwrap()
}

/// Returns the version and build parameters string.
fn version_string() -> String {
    let mut ver = cargo_version();

    let feats = nonstandard_features_string();
    if ! feats.is_empty() {
        ver.push_str(&format!(" [{}]", &feats));
    }

    ver
}

/// Finds whether a feature is enabled by examining the Cargo variable.
fn feature_enabled(name: &str) -> bool {
    env::var(&format!("CARGO_FEATURE_{}", name))
        .map(|e| ! e.is_empty())
        .unwrap_or(false)
}

/// A comma-separated list of non-standard feature choices.
fn nonstandard_features_string() -> String {
    let mut s = Vec::new();

    if ! feature_enabled("WITH_IDNA") {
        s.push("-idna");
    }

    if ! feature_enabled("WITH_TLS") {
        s.push("-tls");
    }

    if ! feature_enabled("WITH_HTTPS") {
        s.push("-https");
    }

    s.join(", ")
}


/// Formats the current date as an ISO 8601 string.
fn build_date() -> String {
    let now = LocalDateTime::now();
    format!("{}", now.date().iso())
}
