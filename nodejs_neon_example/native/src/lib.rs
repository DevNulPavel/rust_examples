#[macro_use]
extern crate neon;

use neon::prelude::*;

fn hello(mut cx: FunctionContext) -> JsResult<JsString> {
    Ok(cx.string("hello node"))
}

fn number_function(mut cx: FunctionContext) -> JsResult<JsNumber> {
    Ok(cx.number(100))
}


// Экспортируем наши Rust функции
register_module!(mut cx, {
    cx.export_function("hello", hello)
});