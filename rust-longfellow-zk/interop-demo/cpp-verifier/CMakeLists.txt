cmake_minimum_required(VERSION 3.16)
project(longfellow_cpp_verifier LANGUAGES CXX)

# Set C++ standard
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Compiler settings
if(CMAKE_CXX_COMPILER_ID STREQUAL "GNU" OR CMAKE_CXX_COMPILER_ID STREQUAL "Clang")
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Wall -Wextra -O3 -march=native")
    set(CMAKE_CXX_FLAGS_DEBUG "-g -O0 -DDEBUG")
    set(CMAKE_CXX_FLAGS_RELEASE "-O3 -DNDEBUG")
endif()

# Include directories
include_directories(${CMAKE_CURRENT_SOURCE_DIR})

# Create the verifier library
add_library(longfellow_verifier STATIC
    cpp_verifier.cpp
    cpp_verifier.hpp
)

# Create the demo executable
add_executable(verify_rust_proof
    verify_rust_proof.cpp
)

# Link the library to the executable
target_link_libraries(verify_rust_proof longfellow_verifier)

# Installation
install(TARGETS longfellow_verifier verify_rust_proof
    LIBRARY DESTINATION lib
    ARCHIVE DESTINATION lib
    RUNTIME DESTINATION bin
)

install(FILES cpp_verifier.hpp
    DESTINATION include
)

# Enable testing
enable_testing()

# Add a simple test
add_test(NAME verify_rust_proof_test
    COMMAND verify_rust_proof --help
)