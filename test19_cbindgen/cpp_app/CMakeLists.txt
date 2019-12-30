cmake_minimum_required(VERSION 3.0.0)
project(cpp_rust_app VERSION 0.1.0)

include_directories("${CMAKE_CURRENT_SOURCE_DIR}/../shared_include/")

if(${CMAKE_BUILD_TYPE} STREQUAL "Release")
    set(LINK_LIBS "${CMAKE_CURRENT_SOURCE_DIR}/../target/release/libtest19_cbindgen.a")
    message("LIBRARY: ${LINK_LIBS}")
elseif(${CMAKE_BUILD_TYPE} STREQUAL "Debug")
    set(LINK_LIBS "${CMAKE_CURRENT_SOURCE_DIR}/../target/debug/libtest19_cbindgen.a")
    message("LIBRARY: ${LINK_LIBS}")
endif()

message("LIBRARY: ${LINK_LIBS}")

add_executable(${PROJECT_NAME} main.cpp)
target_link_libraries(${PROJECT_NAME} ${LINK_LIBS})

set(CPACK_PROJECT_NAME ${PROJECT_NAME})
set(CPACK_PROJECT_VERSION ${PROJECT_VERSION})
include(CPack)
