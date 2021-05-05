mod utils;

use wasm_bindgen::{
    prelude::{
        *
    }
};
use utils::{
    set_panic_hook
};

// Выставляем иной аллокатор если надо
#[cfg(feature = "tiny_allocator")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// Внешние проэкспортированные функции
#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    set_panic_hook();

    #[allow(unused_unsafe)]
    unsafe {
        alert("Hello, test62-wasm!");
    }
}
