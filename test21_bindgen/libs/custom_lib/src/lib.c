#include "lib.h"
#include <stdlib.h>
#include <stdio.h>
#include <string.h>

// Переменная, в которой у нас будет храниться коллбек
static RustCallbackInt32 cbInt32 = NULL;

// Коллбек + объект
static RustCallbackObj cbObj = NULL;
static void* cbObjTarget = NULL;

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

    int32_t register_callback_obj(void* target, RustCallbackObj callback) {
        cbObj = callback;
        cbObjTarget = target;
        return 1;
    }

    void trigger_callback_obj() {
        if(cbObj && cbObjTarget){
            cbObj(cbObjTarget, 7); // Вызовет callback(&rustObject, 7) в Rust
        }
    }

    const char* test_string_code(const char* inputText){
        if (inputText == NULL){
            return "";
        }

        const char* prefix = "PREFIX: ";
        char* newText = malloc(strlen(inputText) + strlen(prefix) + 1);
        sprintf(newText, "%s%s", prefix, inputText);

        return newText;
    }

#ifdef __cplusplus
}
#endif