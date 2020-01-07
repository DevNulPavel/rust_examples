#include "lib.h"

// Описываем тип коллбека
typedef void (*RustCallbackInt32)(int32_t);
typedef void (*RustCallbackObj)(void*, int32_t);

// Переменная, в которой у нас будет храниться коллбек
RustCallbackInt32 cbInt32 = NULL;

// Коллбек + объект
RustCallbackObj cbObj = NULL;
void* cbObjTarget = NULL;

#ifdef __cplusplus
extern "C" {
#endif

    int32_t register_callback_int32(RustCallbackInt32 callback) {
        cbInt32 = callback;
        return 1;
    }

    void trigger_callback_int32() {
        if(cbInt32){
            cbInt32(7); // Вызовет callback(7) в Rust
        }
    }

    int32_t register_callback_obj(void* target, rust_callback callback) {
        cbObj = callback;
        cbObjTarget = target;
        return 1;
    }

    void trigger_callback_obj() {
        if(cbObj && cbObjTarget){
            cbObj(cbObjTarget, 7); // Вызовет callback(&rustObject, 7) в Rust
        }
    }

#ifdef __cplusplus
}
#endif