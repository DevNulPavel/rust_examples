#ifdef NDEBUG
    #undef NDEBUG
    #include <cassert>
#else
    #include <cassert>
#endif

#include <cstdio>
#include <cstring>
#include <cctype>      // tolower
#include <cstdlib>     // malloc, realloc
#include <thread>
#include <unordered_map>
#include <chrono>
#include <iostream>
#include <thread>

// Так как библиотека C-шная, то нужно указывать соглашение о вызовах
// Если используем C++ библиотеку - убрать
// Если в самой библиотеке прописан формат вызова - тоже не нужно
// extern "C" {
    #include <library.h>
// }

int icmp1_C_CODE_V1 (const char* s2, const char* s1 = 0)
{
    thread_local static size_t sz = 32;
    thread_local static char* str = 0;
    
    if (str == 0)
    {
        str = (char*) malloc (sz + 1);
        if (str)
        {
            memset (str, 0, sz + 1);
        }
        else
        {
            return 0;
        }
    }
    
    if (s1)
    {
        size_t sz1 = strlen (s1);
        if (sz1 > sz)
        {
            sz = sz1;
            str = (char*) realloc (str, sz + 1);
        }
        
        if (str)
        {
            char* cp = str;
            while ((*cp++ = (char)tolower(*s1++))) {}
        }
        else
        {
            return 0;
        }
    }
    
    if (str)
    {
        return strcmp (str, s2) == 0 ? 1 : 0;
    }
    
    return 0;
}

int icmp1_C_CODE_V2 (const char* s2, const char* s1 = 0){
    // Буффер для предыдущей строки s1
    thread_local static size_t sz = 32;
    thread_local static char* str = 0;
    
    // Если предудущей строки нету
    if (str == 0){
        // Тогда аллоцируем буффер размером sz + 1, 1 для нулевого символа конца
        str = (char*)malloc (sz + 1);
        // Если аллоцировали - весь массив обнуляем
        if (str){
            memset (str, 0, sz + 1);
        }else{
            return 0;
        }
    }
    
    // Если параметр был передан, тогда
    if (s1){
        // Вычисляем размер переданной строки
        size_t sz1 = strlen (s1);
        // Если размер переданной строки больше, чем прошлый буффер
        if (sz1 > sz){
            // Тогда обновляем значение размера буффера
            sz = sz1;
            // И реаллоцируем наш буффер до большего размера
            str = (char*)realloc (str, sz + 1);
        }
        
        // Если реаллокация прошла без проблем
        if (str){
            // Тогда копируем одну строку в другую посимвольно уменьшая размер
            char* cp = str;
            while (*s1 != 0) {
                char* target = (char*)cp;
                char* source = (char*)s1;

                (*target = (char)tolower(*source));

                cp++;
                s1++;
            }
            // В конце концов выставляем терминальный 0
            *cp = 0;
        }else{
            return 0;
        }
    }
    
    if (str){
        return strcmp(str, s2) == 0 ? 1 : 0;
    }
    
    return 0;
}

int icmp1_C_CODE_V3(const char* s2, const char* s1 = 0)
{
    if(s2 == nullptr){
        return 0;
    }

    struct ThreadStorrage {
        size_t sz;
        char* str;
        
        ThreadStorrage():
            sz(32),
            str(NULL){}
    };
    
#ifndef BUILD_EMSCRIPTEN    
    thread_local static ThreadStorrage storrage;
#else
    static ThreadStorrage storrage;
#endif

/*#ifndef BUILD_EMSCRIPTEN
    static std::mutex mutex;
    static std::unordered_map<std::thread::id, ThreadStorrage> storrages;
    
    mutex.lock();
    ThreadStorrage& storrage = storrages[std::this_thread::get_id()];
    mutex.unlock();
#else
    static ThreadStorrage storrage;
#endif*/
    
    // Если предудущей строки нету
    if (storrage.str == 0){
        // Тогда аллоцируем буффер размером sz + 1, 1 для нулевого символа конца
        storrage.str = (char*)malloc(storrage.sz + 1);
        // Если аллоцировали - весь массив обнуляем
        if (storrage.str){
            memset (storrage.str, 0, storrage.sz + 1);
        }else{
            return 0;
        }
    }
    
    // Если параметр был передан, тогда
    if (s1){
        // Вычисляем размер переданной строки
        size_t sz1 = strlen(s1);
        // Делаем некоторую защиту от кривых входных данных
        if(sz1 > 1024){
            return 0;
        }
        // Если размер переданной строки больше, чем прошлый буффер
        if (sz1 > storrage.sz){
            // Тогда обновляем значение размера буффера
            storrage.sz = sz1;
            // И реаллоцируем наш буффер до большего размера
            storrage.str = (char*)realloc (storrage.str, storrage.sz + 1);
        }
        
        // Если реаллокация прошла без проблем
        if (storrage.str){
            // Тогда копируем одну строку в другую посимвольно уменьшая размер
            char* cp = storrage.str;
            while (*s1 != 0) {
                char* target = (char*)cp;
                char* source = (char*)s1;
                
                (*target = (char)tolower(*source));
                
                cp++;
                s1++;
            }
            // В конце концов выставляем терминальный 0
            *cp = 0;
        }else{
            return 0;
        }
    }
    
    // Делаем некоторую защиту от кривых входных данных
    size_t sz2 = strlen(s2);
    if(sz2 > 1024){
        return 0;
    }

    // Сравниваем
    if (storrage.str){
        return strcmp(storrage.str, s2) == 0 ? 1 : 0;
    }
    
    return 0;
}

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
    /*{
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
        test_raw_pointers();
    }

    {
        test_panic_catch();
    }

    {
        ExpressionFfi* expr = parse_arithmetic("100 + (200*120 + 3)");
        std::printf("Expression: ");
        print_expression(expr);
        std::printf("\n");
        destroy(expr);
    }*/

    const char* invalid_buffer_big = (const char*)malloc(2048);
    const char* invalid_buffer_small = (const char*)malloc(58);

    #define TEST_CODE \
        assert(TEST_FUNC(0, 0) == 0);   \
        assert(TEST_FUNC(0) == 0);      \
        assert(TEST_FUNC(0, "asdsad") == 0);    \
        assert(TEST_FUNC("asdsad") == 0);   \
        assert(TEST_FUNC("asdsad", "asdsad") == 1);     \
        assert(TEST_FUNC("test1", "test1") == 1);   \
        assert(TEST_FUNC("test1", "test1") == 1);   \
        assert(TEST_FUNC("test1", "TEST2") == 0);   \
        assert(TEST_FUNC("test2") == 1);    \
        assert(TEST_FUNC("test0") == 0);    \
        assert(TEST_FUNC("") == 0);     \
        assert(TEST_FUNC("  фывфыв") == 0);     \
        assert(TEST_FUNC("asdasd", "ASDASD") == 1);     \
        assert(TEST_FUNC("asda", "ASDASD") == 0);   \
        assert(TEST_FUNC("as", "ASD") == 0);    \
        assert(TEST_FUNC("as", "ASDDADASDS") == 0);     \
        assert(TEST_FUNC("asddadasds", "ASDDADASDS") == 1);     \
        assert(TEST_FUNC("add", "ASDDADASDS") == 0);    \
        assert(TEST_FUNC("add") == 0);      \
        assert(TEST_FUNC("asddadasds") == 1);   \
        assert(TEST_FUNC("asd", "ASD") == 1);   \
        assert(TEST_FUNC("asd") == 1);      \
        assert(TEST_FUNC("asd_") == 0);     \
        assert(TEST_FUNC("asd____") == 0);      \
        assert(TEST_FUNC("a") == 0);    \
        assert(TEST_FUNC("") == 0);     \
        assert(TEST_FUNC(invalid_buffer_big, "asads") == 0);    \
        assert(TEST_FUNC("ASD", invalid_buffer_big) == 0);      \
        assert(TEST_FUNC("asdsd") == 0);    \
        assert(TEST_FUNC(invalid_buffer_small, "asads") == 0);      \
        assert(TEST_FUNC("ASD", invalid_buffer_small) == 0);    \
        assert(TEST_FUNC("asdsd") == 0);

    // Некий аналог unit тестов
    /*#define TEST_FUNC(...) icmp1_C_CODE_V1(__VA_ARGS__)
    {
        auto start = std::chrono::high_resolution_clock::now();
        auto f = [&](){
            for(size_t i = 0; i < 100000; i++){
                TEST_CODE
            }
        };
        std::thread t1(f);
        std::thread t2(f);
        std::thread t3(f);
        t1.join();
        t2.join();
        t3.join();
        auto finish = std::chrono::high_resolution_clock::now();
        std::chrono::duration<double> elapsed = finish - start;
        std::cout << "V1 elapsed time: " << elapsed.count() << " s\n";
    }
    #undef TEST_FUNC

    #define TEST_FUNC(...) icmp1_C_CODE_V2(__VA_ARGS__)
    {
        auto start = std::chrono::high_resolution_clock::now();
        auto f = [&](){
            for(size_t i = 0; i < 100000; i++){
                TEST_CODE              
            }
        };
        std::thread t1(f);
        std::thread t2(f);
        std::thread t3(f);
        t1.join();
        t2.join();
        t3.join();
        auto finish = std::chrono::high_resolution_clock::now();
        std::chrono::duration<double> elapsed = finish - start;
        std::cout << "V2 elapsed time: " << elapsed.count() << " s\n";
    }
    #undef TEST_FUNC*/

    #define TEST_FUNC(...) icmp1_C_CODE_V3(__VA_ARGS__)
    {
        auto start = std::chrono::high_resolution_clock::now();
        auto f = [&](){
            for(size_t i = 0; i < 100000; i++){
                TEST_CODE
            }
        };
        std::thread t1(f);
        std::thread t2(f);
        std::thread t3(f);
        t1.join();
        t2.join();
        t3.join();
        auto finish = std::chrono::high_resolution_clock::now();
        std::chrono::duration<double> elapsed = finish - start;
        std::cout << "V3 elapsed time: " << elapsed.count() << " s\n";
    }
    #undef TEST_FUNC


    #define TEST_FUNC(...) icmp1_RUST_CODE(__VA_ARGS__)
    {
        // Сравнение времени исполнения
        auto start = std::chrono::high_resolution_clock::now();
        auto f = [&](){
            for(size_t i = 0; i < 100000; i++){
                TEST_CODE             
            }
        };
        std::thread t1(f);
        std::thread t2(f);
        std::thread t3(f);
        t1.join();
        t2.join();
        t3.join();
        auto finish = std::chrono::high_resolution_clock::now();
        std::chrono::duration<double> elapsed = finish - start;
        std::cout << "Rust elapsed time: " << elapsed.count() << " s\n";        
    }
    #undef TEST_FUNC



    return 0;
}
