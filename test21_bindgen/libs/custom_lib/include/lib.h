#pragma once

#include <stdlib.h>
#include <inttypes.h>

// Описываем тип коллбека
typedef void (*rust_callback)(int32_t);

#ifdef __cplusplus
extern "C" {
#endif

    int32_t register_callback_int32(rust_callback callback);
    void trigger_callback_int32();
    int32_t register_callback_obj(void* target, rust_callback callback);
    void trigger_callback_obj();

#ifdef __cplusplus
}
#endif
