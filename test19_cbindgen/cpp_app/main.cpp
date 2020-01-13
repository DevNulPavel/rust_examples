#include <cstdio>
#include <cstring>

// Так как библиотека C-шная, то нужно указывать соглашение о вызовах
// Если используем C++ библиотеку - убрать
// Если в самой библиотеке прописан формат вызова - тоже не нужно
// extern "C" {
    #include <library.h>
// }

void print_expression(ExpressionFfi* expression) {
    switch (expression->expression_type) {
        case Add:
        case Subtract:
        case Multiply:
        case Divide:{
            const char* operations[] = {"+", "-", "*", "/"};
            std::printf("(");
            print_expression(expression->data.pair_operands.left);
            std::printf("%s", operations[expression->expression_type]);
            print_expression(expression->data.pair_operands.right);
            std::printf(")");
        }break;

        case UnaryMinus:{
            std::printf("-");
            print_expression(expression->data.single_operand);
        }break;

        case Value:{
            std::printf("%lld", expression->data.value);
        }break;
    }
}

int main(int argc, char const *argv[]){
    {
        int32_t result = function_1(10);
        std::printf("Value from RUST: %d\n", result);
    }

    {
        Buffer_i32 buffer;
        std::memset(buffer.data, 0, sizeof(buffer.data));
        buffer.data[0] = 10;
        buffer.data[1] = 20;
        buffer.data[2] = 20;
        buffer.len = 3;
        int32_t result = function_2(buffer);
        std::printf("Value from RUST: %d\n", result);
    }

    {
        ExpressionFfi* expr = parse_arithmetic("100 + (200*120 + 3)");
        std::printf("Expression: ");
        print_expression(expr);
        std::printf("\n");
        destroy(expr);
    }
    return 0;
}
