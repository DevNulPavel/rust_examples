#ifndef RUST_LIBRARY
#define RUST_LIBRARY

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

struct ExpressionFfi;
struct PairOperands;

typedef enum ExpressionType {
    Add = 0,
    Subtract = 1,
    Multiply = 2,
    Divide = 3,
    UnaryMinus = 4,
    Value = 5,
} ExpressionType;

typedef struct PairOperands {
    struct ExpressionFfi *left;
    struct ExpressionFfi *right;
} PairOperands;

typedef union ExpressionData {
    struct PairOperands pair_operands;
    struct ExpressionFfi *single_operand;
    int64_t value;
} ExpressionData;

typedef struct ExpressionFfi {
    enum ExpressionType expression_type;
    union ExpressionData data;
} ExpressionFfi;

typedef struct Buffer_i32 {
    int32_t data[8];
    uintptr_t len;
} Buffer_i32;

#ifdef __cplusplus
extern "C" {
#endif
    int32_t function_1(int32_t param);
    int32_t function_2(Buffer_i32 buffer);
    void test_raw_pointers();
    
    ExpressionFfi *parse_arithmetic(const char *s);
    void destroy(ExpressionFfi *expression);
    
#ifdef __cplusplus
}
#endif

#endif
