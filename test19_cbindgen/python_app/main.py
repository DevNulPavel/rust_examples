#! /usr/bin/env python3

import sys
import os
import os.path
import ctypes
import cffi

SCRIPT_FOLDER = os.path.dirname(os.path.realpath(__file__))

def test_cdll():
    if(len(sys.argv) < 2):
        print("Lib type is missing")
        exit(0)

    lib_type = sys.argv[1][1:]

    if sys.platform == "darwin":
        prefix = "lib"
        ext = "dylib"
    elif sys.platform == "win32":
        prefix = ""
        ext = "dll"
    else:
        prefix = "lib"
        ext = "so"

    lib_path = os.path.join(SCRIPT_FOLDER, "../target/{}/".format(lib_type), "{}test19_cbindgen.{}".format(prefix, ext))
    lib_path = os.path.abspath(lib_path)

    lib = ctypes.cdll.LoadLibrary(lib_path)

    function_1 = lib.function_1

    input = 4
    output = function_1(input)
    print('{} = {}'.format(input, output))


def test_ffi():
    pass


def main():
    # test_cdll()
    test_ffi()


if __name__ == "__main__":
    main()