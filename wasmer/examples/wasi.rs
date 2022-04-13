//! Running a WASI compiled WebAssembly module with Wasmer.
//!
//! This example illustrates how to run WASI modules with
//! Wasmer. To run WASI we have to have to do mainly 3 steps:
//!
//!   1. Create a `WasiEnv` instance
//!   2. Attach the imports from the `WasiEnv` to a new instance
//!   3. Run the `WASI` module.
//!
//! You can run the example directly by executing in Wasmer root:
//!
//! ```shell
//! cargo run --example wasi --release --features "cranelift,wasi"
//! ```
//!
//! Ready?

use wasmer::{Instance, Module, Store};
use wasmer_compiler_cranelift::Cranelift;
use wasmer_engine_universal::Universal;
use wasmer_wasi::WasiState;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wasm_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/wasi-wast/wasi/unstable/hello.wasm"
    );
    // Let's declare the Wasm module with the text representation.
    let wasm_bytes = std::fs::read(wasm_path)?;

    // Create a Store.
    // Note that we don't need to specify the engine/compiler if we want to use
    // the default provided by Wasmer.
    // You can use `Store::default()` for that.
    let store = Store::new(&Universal::new(Cranelift::default()).engine());

    println!("Compiling module...");
    // Let's compile the Wasm module.
    let module = Module::new(&store, wasm_bytes)?;

    println!("Creating `WasiEnv`...");
    // First, we create the `WasiEnv`
    let mut wasi_env = WasiState::new("hello")
        // .args(&["world"])
        // .env("KEY", "Value")
        .finalize()?;

    println!("Instantiating module with WASI imports...");
    // Then, we get the import object related to our WASI
    // and attach it to the Wasm instance.
    let import_object = wasi_env.import_object(&module)?;
    let instance = Instance::new(&module, &import_object)?;

    println!("Call WASI `_start` function...");
    // And we just call the `_start` function!
    let start = instance.exports.get_function("_start")?;
    start.call(&[])?;

    Ok(())
}

#[test]
#[cfg(feature = "wasi")]
fn test_wasi() -> Result<(), Box<dyn std::error::Error>> {
    main()
}
