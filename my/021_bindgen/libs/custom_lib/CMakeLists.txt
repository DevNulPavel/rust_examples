cmake_minimum_required(VERSION 3.1)
set(PROJECT_NAME custom_lib)

project(${PROJECT_NAME} LANGUAGES C) # CXX

# if(NOT CMAKE_CXX_STANDARD)
#     set(CMAKE_CXX_STANDARD 11)
#     set(CMAKE_CXX_STANDARD_REQUIRED ON)
#     set(CMAKE_CXX_EXTENSIONS OFF)
# endif(NOT CMAKE_CXX_STANDARD)

set(CMAKE_C_FLAGS ${CMAKE_C_FLAGS} -fPIC)
set(CMAKE_CXX_FLAGS ${CMAKE_CXX_FLAGS} -fPIC)

include_directories("${CMAKE_CURRENT_SOURCE_DIR}/include")

add_library(${PROJECT_NAME} STATIC "src/lib.c")