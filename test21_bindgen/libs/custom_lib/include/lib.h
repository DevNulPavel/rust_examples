#pragma once

#include <inttypes.h>

// Описываем тип коллбека
typedef void (*RustCallbackInt32)(int32_t);
typedef void (*RustCallbackObj)(void*, int32_t);

#ifdef __cplusplus
extern "C" {
#endif

    int32_t register_callback_int32(RustCallbackInt32 callback);
    void trigger_callback_int32();
    int32_t register_callback_obj(void* target, RustCallbackObj callback);
    void trigger_callback_obj();
    const char* test_string_code(const char* inputText);

#ifdef __cplusplus
}
#endif
