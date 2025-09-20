cmake_minimum_required(VERSION 3.10)
project(kadugu_net_examples C)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)

# Set output directories
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin)
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)

# Find the Rust library
find_library(KADUGU_NET_LIB kadugu_net HINTS ${CMAKE_SOURCE_DIR}/target/debug ${CMAKE_SOURCE_DIR}/target/release)
if(NOT KADUGU_NET_LIB)
    message(FATAL_ERROR "Could not find kadugu_net library. Please build the Rust library first.")
endif()

# Add the library include directory
include_directories(${CMAKE_SOURCE_DIR})

# Add the C wrapper library
add_library(kadugu_net_wrapper SHARED
    kadugu_net.c
)

# Link the Rust library
target_link_libraries(kadugu_net_wrapper ${KADUGU_NET_LIB})

# Add examples
add_executable(kadugu_client_example examples/client_example.c)
target_link_libraries(kadugu_client_example kadugu_net_wrapper)

add_executable(kadugu_server_example examples/server_example.c)
target_link_libraries(kadugu_server_example kadugu_net_wrapper)

# Copy the Rust DLL to the output directory
add_custom_command(TARGET kadugu_net_wrapper POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
    ${KADUGU_NET_LIB} $<TARGET_FILE_DIR:kadugu_net_wrapper>
    COMMENT "Copying Rust library"
)
