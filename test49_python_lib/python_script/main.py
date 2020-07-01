#! /usr/bin/env python3

import rust_python_lib

def main():
    result = rust_python_lib.sum_as_string(10, 15)
    print(result)

if __name__ == "__main__":
    main()