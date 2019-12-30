#include <library.h>
#include <cstdio>

int main(int argc, char const *argv[]){    
    int32_t result = function_1(10);
    std::printf("Value from RUST: %d", result);
    return 0;
}
