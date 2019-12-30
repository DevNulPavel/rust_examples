#include <library.h>
#include <cstdio>
#include <cstring>

int main(int argc, char const *argv[]){
    {
        int32_t result = rust_lib::function_1(10);
        std::printf("Value from RUST: %d\n", result);
    }

    {
        rust_lib::Buffer<int32_t> buffer;
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
