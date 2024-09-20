extern crate pollster as reexported_pollster;

use std::future::ready;

#[pollster::main]
async fn main_basic() {
    ready(42).await;
}

#[test]
fn basic() {
    main_basic();
}

#[pollster::main]
async fn main_result() -> Result<(), std::io::Error> {
    ready(42).await;
    Ok(())
}

#[test]
fn result() {
    main_result().unwrap();
}

#[pollster::main(crate = reexported_pollster)]
async fn main_crate_path() {
    ready(42).await;
}

#[pollster::main(crate = "reexported_pollster")]
async fn main_crate_str() {
    ready(42).await;
}

#[test]
fn crate_() {
    main_crate_path();
    main_crate_str();
}
