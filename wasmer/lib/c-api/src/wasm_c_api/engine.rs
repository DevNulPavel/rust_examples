#[cfg(feature = "compiler")]
pub use super::unstable::engine::wasmer_is_compiler_available;
pub use super::unstable::engine::{
    wasm_config_set_features, wasm_config_set_target, wasmer_is_engine_available,
};
use super::unstable::features::wasmer_features_t;
#[cfg(feature = "middlewares")]
pub use super::unstable::middlewares::wasm_config_push_middleware;
#[cfg(feature = "middlewares")]
use super::unstable::middlewares::wasmer_middleware_t;
use super::unstable::target_lexicon::wasmer_target_t;
use crate::error::update_last_error;
use cfg_if::cfg_if;
use std::sync::Arc;
use wasmer_api::Engine;
#[cfg(feature = "dylib")]
use wasmer_engine_dylib::Dylib;
#[cfg(feature = "staticlib")]
use wasmer_engine_staticlib::Staticlib;
#[cfg(feature = "universal")]
use wasmer_engine_universal::Universal;

/// Kind of compilers that can be used by the engines.
///
/// This is a Wasmer-specific type with Wasmer-specific functions for
/// manipulating it.
#[cfg(feature = "compiler")]
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub enum wasmer_compiler_t {
    /// Variant to represent the Cranelift compiler. See the
    /// [`wasmer_compiler_cranelift`] Rust crate.
    CRANELIFT = 0,

    /// Variant to represent the LLVM compiler. See the
    /// [`wasmer_compiler_llvm`] Rust crate.
    LLVM = 1,

    /// Variant to represent the Singlepass compiler. See the
    /// [`wasmer_compiler_singlepass`] Rust crate.
    SINGLEPASS = 2,
}

#[cfg(feature = "compiler")]
impl Default for wasmer_compiler_t {
    fn default() -> Self {
        cfg_if! {
            if #[cfg(feature = "cranelift")] {
                Self::CRANELIFT
            } else if #[cfg(feature = "llvm")] {
                Self::LLVM
            } else if #[cfg(feature = "singlepass")] {
                Self::SINGLEPASS
            } else {
                compile_error!("Please enable one of the compiler backends")
            }
        }
    }
}

/// Kind of engines that can be used by the store.
///
/// This is a Wasmer-specific type with Wasmer-specific functions for
/// manipulating it.
#[derive(Debug, Copy, Clone)]
#[repr(C)]
#[allow(non_camel_case_types)]
pub enum wasmer_engine_t {
    /// Variant to represent the Universal engine. See the
    /// [`wasmer_engine_universal`] Rust crate.
    UNIVERSAL = 0,

    /// Variant to represent the Dylib engine. See the
    /// [`wasmer_engine_dylib`] Rust crate.
    DYLIB = 1,

    /// Variant to represent the Staticlib engine. See the
    /// [`wasmer_engine_staticlib`] Rust crate.
    STATICLIB = 2,
}

impl Default for wasmer_engine_t {
    fn default() -> Self {
        cfg_if! {
            if #[cfg(feature = "universal")] {
                Self::UNIVERSAL
            } else if #[cfg(feature = "dylib")] {
                Self::DYLIB
            } else if #[cfg(feature = "staticlib")] {
                Self::STATICLIB
            } else {
                compile_error!("Please enable one of the engines")
            }
        }
    }
}

/// A configuration holds the compiler and the engine used by the store.
///
/// cbindgen:ignore
#[derive(Debug, Default)]
#[repr(C)]
pub struct wasm_config_t {
    engine: wasmer_engine_t,
    #[cfg(feature = "compiler")]
    compiler: wasmer_compiler_t,
    #[cfg(feature = "middlewares")]
    pub(super) middlewares: Vec<wasmer_middleware_t>,
    pub(super) nan_canonicalization: bool,
    pub(super) features: Option<Box<wasmer_features_t>>,
    pub(super) target: Option<Box<wasmer_target_t>>,
}

/// Create a new default Wasmer configuration.
///
/// # Example
///
/// ```rust
/// # use inline_c::assert_c;
/// # fn main() {
/// #    (assert_c! {
/// # #include "tests/wasmer.h"
/// #
/// int main() {
///     // Create the configuration.
///     wasm_config_t* config = wasm_config_new();
///
///     // Create the engine.
///     wasm_engine_t* engine = wasm_engine_new_with_config(config);
///
///     // Check we have an engine!
///     assert(engine);
///
///     // Free everything.
///     wasm_engine_delete(engine);
///
///     return 0;
/// }
/// #    })
/// #    .success();
/// # }
/// ```
///
/// cbindgen:ignore
#[no_mangle]
pub extern "C" fn wasm_config_new() -> Box<wasm_config_t> {
    Box::new(wasm_config_t::default())
}

/// Delete a Wasmer config object.
///
/// This function does not need to be called if `wasm_engine_new_with_config` or
/// another function that takes ownership of the `wasm_config_t` is called.
///
/// # Example
///
/// ```rust
/// # use inline_c::assert_c;
/// # fn main() {
/// #    (assert_c! {
/// # #include "tests/wasmer.h"
/// #
/// int main() {
///     // Create the configuration.
///     wasm_config_t* config = wasm_config_new();
///
///     // Delete the configuration
///     wasm_config_delete(config);
///
///     return 0;
/// }
/// #    })
/// #    .success();
/// # }
/// ```
/// cbindgen:ignore
#[no_mangle]
pub extern "C" fn wasm_config_delete(_config: Option<Box<wasm_config_t>>) {}

/// Updates the configuration to specify a particular compiler to use.
///
/// This is a Wasmer-specific function.
///
/// # Example
///
/// ```rust
/// # use inline_c::assert_c;
/// # fn main() {
/// #    (assert_c! {
/// # #include "tests/wasmer.h"
/// #
/// int main() {
///     // Create the configuration.
///     wasm_config_t* config = wasm_config_new();
///
///     // Use the Cranelift compiler, if available.
///     if (wasmer_is_compiler_available(CRANELIFT)) {
///         wasm_config_set_compiler(config, CRANELIFT);
///     }
///     // Or maybe LLVM?
///     else if (wasmer_is_compiler_available(LLVM)) {
///         wasm_config_set_compiler(config, LLVM);
///     }
///     // Or maybe Singlepass?
///     else if (wasmer_is_compiler_available(SINGLEPASS)) {
///         wasm_config_set_compiler(config, SINGLEPASS);
///     }
///     // OK, let's run with no particular compiler.
///
///     // Create the engine.
///     wasm_engine_t* engine = wasm_engine_new_with_config(config);
///
///     // Check we have an engine!
///     assert(engine);
///
///     // Free everything.
///     wasm_engine_delete(engine);
///
///     return 0;
/// }
/// #    })
/// #    .success();
/// # }
/// ```
#[cfg(feature = "compiler")]
#[no_mangle]
pub extern "C" fn wasm_config_set_compiler(
    config: &mut wasm_config_t,
    compiler: wasmer_compiler_t,
) {
    config.compiler = compiler;
}

/// Updates the configuration to specify a particular engine to use.
///
/// This is a Wasmer-specific function.
///
/// # Example
///
/// ```rust
/// # use inline_c::assert_c;
/// # fn main() {
/// #    (assert_c! {
/// # #include "tests/wasmer.h"
/// #
/// int main() {
///     // Create the configuration.
///     wasm_config_t* config = wasm_config_new();
///
///     // Use the Universal engine, if available.
///     if (wasmer_is_engine_available(UNIVERSAL)) {
///         wasm_config_set_engine(config, UNIVERSAL);
///     }
///     // Or maybe the Dylib engine?
///     else if (wasmer_is_engine_available(DYLIB)) {
///         wasm_config_set_engine(config, DYLIB);
///     }
///     // OK, let's do not specify any particular engine.
///
///     // Create the engine.
///     wasm_engine_t* engine = wasm_engine_new_with_config(config);
///
///     // Check we have an engine!
///     assert(engine);
///
///     // Free everything.
///     wasm_engine_delete(engine);
///
///     return 0;
/// }
/// #    })
/// #    .success();
/// # }
/// ```
#[no_mangle]
pub extern "C" fn wasm_config_set_engine(config: &mut wasm_config_t, engine: wasmer_engine_t) {
    config.engine = engine;
}

/// An engine is used by the store to drive the compilation and the
/// execution of a WebAssembly module.
///
/// cbindgen:ignore
#[repr(C)]
pub struct wasm_engine_t {
    pub(crate) inner: Arc<dyn Engine + Send + Sync>,
}

#[cfg(feature = "compiler")]
use wasmer_api::CompilerConfig;

#[cfg(feature = "compiler")]
fn get_default_compiler_config() -> Box<dyn CompilerConfig> {
    cfg_if! {
        if #[cfg(feature = "cranelift")] {
            Box::new(wasmer_compiler_cranelift::Cranelift::default())
        } else if #[cfg(feature = "llvm")] {
            Box::new(wasmer_compiler_llvm::LLVM::default())
        } else if #[cfg(feature = "singlepass")] {
            Box::new(wasmer_compiler_singlepass::Singlepass::default())
        } else {
            compile_error!("Please enable one of the compiler backends")
        }
    }
}

cfg_if! {
    if #[cfg(all(feature = "universal", feature = "compiler"))] {
        /// Creates a new Universal engine with the default compiler.
        ///
        /// # Example
        ///
        /// See [`wasm_engine_delete`].
        ///
        /// cbindgen:ignore
        #[no_mangle]
        pub extern "C" fn wasm_engine_new() -> Box<wasm_engine_t> {
            let compiler_config: Box<dyn CompilerConfig> = get_default_compiler_config();
            let engine: Arc<dyn Engine + Send + Sync> = Arc::new(Universal::new(compiler_config).engine());
            Box::new(wasm_engine_t { inner: engine })
        }
    } else if #[cfg(feature = "universal")] {
        /// Creates a new headless Universal engine.
        ///
        /// # Example
        ///
        /// See [`wasm_engine_delete`].
        ///
        /// cbindgen:ignore
        #[no_mangle]
        pub extern "C" fn wasm_engine_new() -> Box<wasm_engine_t> {
            let engine: Arc<dyn Engine + Send + Sync> = Arc::new(Universal::headless().engine());
            Box::new(wasm_engine_t { inner: engine })
        }
    } else if #[cfg(all(feature = "dylib", feature = "compiler"))] {
        /// Creates a new Dylib engine with the default compiler.
        ///
        /// # Example
        ///
        /// See [`wasm_engine_delete`].
        ///
        /// cbindgen:ignore
        #[no_mangle]
        pub extern "C" fn wasm_engine_new() -> Box<wasm_engine_t> {
            let compiler_config: Box<dyn CompilerConfig> = get_default_compiler_config();
            let engine: Arc<dyn Engine + Send + Sync> = Arc::new(Dylib::new(compiler_config).engine());
            Box::new(wasm_engine_t { inner: engine })
        }
    } else if #[cfg(feature = "dylib")] {
        /// Creates a new headless Dylib engine.
        ///
        /// # Example
        ///
        /// See [`wasm_engine_delete`].
        ///
        /// cbindgen:ignore
        #[no_mangle]
        pub extern "C" fn wasm_engine_new() -> Box<wasm_engine_t> {
            let engine: Arc<dyn Engine + Send + Sync> = Arc::new(Dylib::headless().engine());
            Box::new(wasm_engine_t { inner: engine })
        }
    }
    // There are currently no uses of the Staticlib engine + compiler from the C API.
    // So if we get here, we default to headless mode regardless of if `compiler` is enabled.
    else if #[cfg(feature = "staticlib")] {
        /// Creates a new headless Staticlib engine.
        ///
        /// # Example
        ///
        /// See [`wasm_engine_delete`].
        ///
        /// cbindgen:ignore
        #[no_mangle]
        pub extern "C" fn wasm_engine_new() -> Box<wasm_engine_t> {
            let engine: Arc<dyn Engine + Send + Sync> = Arc::new(Staticlib::headless().engine());
            Box::new(wasm_engine_t { inner: engine })
        }
    } else {
        /// Creates a new unknown engine, i.e. it will panic with an error message.
        ///
        /// # Example
        ///
        /// See [`wasm_engine_delete`].
        ///
        /// cbindgen:ignore
        #[no_mangle]
        pub extern "C" fn wasm_engine_new() -> Box<wasm_engine_t> {
            unimplemented!("No engine attached; You might want to recompile `wasmer_c_api` with for example `--feature universal`");
        }
    }
}

/// Deletes an engine.
///
/// # Example
///
/// ```rust
/// # use inline_c::assert_c;
/// # fn main() {
/// #    (assert_c! {
/// # #include "tests/wasmer.h"
/// #
/// int main() {
///     // Create a default engine.
///     wasm_engine_t* engine = wasm_engine_new();
///
///     // Check we have an engine!
///     assert(engine);
///
///     // Free everything.
///     wasm_engine_delete(engine);
///
///     return 0;
/// }
/// #    })
/// #    .success();
/// # }
/// ```
///
/// cbindgen:ignore
#[no_mangle]
pub unsafe extern "C" fn wasm_engine_delete(_engine: Option<Box<wasm_engine_t>>) {}

/// Creates an engine with a particular configuration.
///
/// # Example
///
/// See [`wasm_config_new`].
///
/// cbindgen:ignore
#[no_mangle]
pub extern "C" fn wasm_engine_new_with_config(
    config: Option<Box<wasm_config_t>>,
) -> Option<Box<wasm_engine_t>> {
    #[allow(dead_code)]
    fn return_with_error(msg: &str) -> Option<Box<wasm_engine_t>> {
        update_last_error(msg);

        return None;
    }

    let config = config?;

    cfg_if! {
        if #[cfg(feature = "compiler")] {
            #[allow(unused_mut)]
            let mut compiler_config: Box<dyn CompilerConfig> = match config.compiler {
                wasmer_compiler_t::CRANELIFT => {
                    cfg_if! {
                        if #[cfg(feature = "cranelift")] {
                            Box::new(wasmer_compiler_cranelift::Cranelift::default())
                        } else {
                            return return_with_error("Wasmer has not been compiled with the `cranelift` feature.");
                        }
                    }
                },
                wasmer_compiler_t::LLVM => {
                    cfg_if! {
                        if #[cfg(feature = "llvm")] {
                            Box::new(wasmer_compiler_llvm::LLVM::default())
                        } else {
                            return return_with_error("Wasmer has not been compiled with the `llvm` feature.");
                        }
                    }
                },
                wasmer_compiler_t::SINGLEPASS => {
                    cfg_if! {
                        if #[cfg(feature = "singlepass")] {
                            Box::new(wasmer_compiler_singlepass::Singlepass::default())
                        } else {
                            return return_with_error("Wasmer has not been compiled with the `singlepass` feature.");
                        }
                    }
                },
            };

            #[cfg(feature = "middlewares")]
            for middleware in config.middlewares {
                compiler_config.push_middleware(middleware.inner);
            }

            if config.nan_canonicalization {
                compiler_config.canonicalize_nans(true);
            }

            let inner: Arc<dyn Engine + Send + Sync> = match config.engine {
                wasmer_engine_t::UNIVERSAL => {
                    cfg_if! {
                        if #[cfg(feature = "universal")] {
                            let mut builder = Universal::new(compiler_config);

                            if let Some(target) = config.target {
                                builder = builder.target(target.inner);
                            }

                            if let Some(features) = config.features {
                                builder = builder.features(features.inner);
                            }

                            Arc::new(builder.engine())
                        } else {
                            return return_with_error("Wasmer has not been compiled with the `universal` feature.");
                        }
                    }
                },
                wasmer_engine_t::DYLIB => {
                    cfg_if! {
                        if #[cfg(feature = "dylib")] {
                            let mut builder = Dylib::new(compiler_config);

                            if let Some(target) = config.target {
                                builder = builder.target(target.inner);
                            }

                            if let Some(features) = config.features {
                                builder = builder.features(features.inner);
                            }

                            Arc::new(builder.engine())
                        } else {
                            return return_with_error("Wasmer has not been compiled with the `dylib` feature.");
                        }
                    }
                },
                wasmer_engine_t::STATICLIB => {
                    cfg_if! {
                        // There are currently no uses of the Staticlib engine + compiler from the C API.
                        // So we run in headless mode.
                        if #[cfg(feature = "staticlib")] {
                            let mut builder = Staticlib::headless();

                            if let Some(target) = config.target {
                                builder = builder.target(target.inner);
                            }

                            if let Some(features) = config.features {
                                builder = builder.features(features.inner);
                            }

                            Arc::new(builder.engine())
                        } else {
                            return return_with_error("Wasmer has not been compiled with the `staticlib` feature.");
                        }
                    }
                },
            };
            Some(Box::new(wasm_engine_t { inner }))
        } else {
            let inner: Arc<dyn Engine + Send + Sync> = match config.engine {
                wasmer_engine_t::UNIVERSAL => {
                    cfg_if! {
                        if #[cfg(feature = "universal")] {
                            let mut builder = Universal::headless();

                            if let Some(target) = config.target {
                                builder = builder.target(target.inner);
                            }

                            if let Some(features) = config.features {
                                builder = builder.features(features.inner);
                            }

                            Arc::new(builder.engine())
                        } else {
                            return return_with_error("Wasmer has not been compiled with the `universal` feature.");
                        }
                    }
                },
                wasmer_engine_t::DYLIB => {
                    cfg_if! {
                        if #[cfg(feature = "dylib")] {
                            let mut builder = Dylib::headless();

                            if let Some(target) = config.target {
                                builder = builder.target(target.inner);
                            }

                            if let Some(features) = config.features {
                                builder = builder.features(features.inner);
                            }

                            Arc::new(builder.engine())
                        } else {
                            return return_with_error("Wasmer has not been compiled with the `dylib` feature.");
                        }
                    }
                },
                wasmer_engine_t::STATICLIB => {
                    cfg_if! {
                        if #[cfg(feature = "staticlib")] {
                            let mut builder = Staticlib::headless();

                            if let Some(target) = config.target {
                                builder = builder.target(target.inner);
                            }

                            if let Some(features) = config.features {
                                builder = builder.features(features.inner);
                            }

                            Arc::new(builder.engine())
                        } else {
                            return return_with_error("Wasmer has not been compiled with the `staticlib` feature.");
                        }
                    }
                },
            };
            Some(Box::new(wasm_engine_t { inner }))
        }
    }
}

#[cfg(test)]
mod tests {
    use inline_c::assert_c;

    #[test]
    fn test_engine_new() {
        (assert_c! {
            #include "tests/wasmer.h"

            int main() {
                wasm_engine_t* engine = wasm_engine_new();
                assert(engine);

                wasm_engine_delete(engine);

                return 0;
            }
        })
        .success();
    }
}
