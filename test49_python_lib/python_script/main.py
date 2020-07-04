#! /usr/bin/env python3

import rust_python_lib

class Test:
    def __init__(self):
        pass

def main():
    result = rust_python_lib.sum_as_string_1(10, 15)
    print(result)
    
    result = rust_python_lib.num_kwds(param1=123, param2=123, param3=123)
    print("Params count: {}".format(result))

    result = rust_python_lib.add(123, b=123)
    print("Add result: {}".format(result))

    obj = rust_python_lib.MyClass(10, "Text")
    result = obj.my_method(1, 2)
    print("Class result: {}".format(result))

    result = rust_python_lib.search_using_threads("asd asd asd\nasd\nsd\nsd\nsd", "sd")
    print("Search result: {}".format(result))

if __name__ == "__main__":
    main()