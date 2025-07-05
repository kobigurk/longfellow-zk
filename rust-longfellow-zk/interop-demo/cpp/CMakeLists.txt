cmake_minimum_required(VERSION 3.10)
project(LongfellowCppVerifier)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Find required packages
find_package(OpenSSL REQUIRED)
find_package(Threads REQUIRED)

# Add the Rust library path
set(RUST_LIB_PATH "${CMAKE_CURRENT_SOURCE_DIR}/../../target/release")

# Create the verifier library
add_library(longfellow_verifier STATIC
    longfellow_verifier.cpp
)

target_include_directories(longfellow_verifier PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}
    ${OPENSSL_INCLUDE_DIR}
)

target_link_libraries(longfellow_verifier
    ${OPENSSL_LIBRARIES}
    ${CMAKE_THREAD_LIBS_INIT}
)

# Create the example executable
add_executable(example_verifier
    example_verifier.cpp
)

target_link_libraries(example_verifier
    longfellow_verifier
    ${RUST_LIB_PATH}/liblongfellow_interop.a
    dl
    m
)

# Add compiler flags
if(CMAKE_COMPILER_IS_GNUCXX OR CMAKE_CXX_COMPILER_ID MATCHES "Clang")
    target_compile_options(longfellow_verifier PRIVATE -Wall -Wextra -O3)
    target_compile_options(example_verifier PRIVATE -Wall -Wextra -O3)
endif()

# Installation rules
install(TARGETS longfellow_verifier
    ARCHIVE DESTINATION lib
    LIBRARY DESTINATION lib
)

install(FILES longfellow_verifier.hpp
    DESTINATION include/longfellow
)

# Add tests
enable_testing()

add_executable(test_field_ops test_field_ops.cpp)
target_link_libraries(test_field_ops longfellow_verifier)
add_test(NAME FieldOperations COMMAND test_field_ops)

# Package configuration
include(CMakePackageConfigHelpers)

configure_package_config_file(
    "${CMAKE_CURRENT_SOURCE_DIR}/LongfellowConfig.cmake.in"
    "${CMAKE_CURRENT_BINARY_DIR}/LongfellowConfig.cmake"
    INSTALL_DESTINATION lib/cmake/Longfellow
)

install(FILES
    "${CMAKE_CURRENT_BINARY_DIR}/LongfellowConfig.cmake"
    DESTINATION lib/cmake/Longfellow
)