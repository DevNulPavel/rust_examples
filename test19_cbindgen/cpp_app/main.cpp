#include <cstdio>
#include <cstring>

// Так как библиотека C-шная, то нужно указывать соглашение о вызовах
// Если используем C++ библиотеку - убрать
// Если в самой библиотеке прописан формат вызова - тоже не нужно
// extern "C" {
    #include <library.h>
// }

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
    return 0;
}
