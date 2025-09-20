#include "kadugu_net.h"
#include <windows.h>
#include <stdio.h>

// Forward declarations for the Rust functions
typedef void* (*start_client_fn)(const char*, const char*);
typedef void (*stop_client_fn)(void*);
typedef void* (*start_server_fn)(const char*, const char*);
typedef void (*stop_server_fn)(void*);
typedef char* (*get_server_peer_id_fn)(void*);
typedef void (*free_string_fn)(char*);

// Function pointers to hold the loaded functions
static start_client_fn start_client_ptr = NULL;
static stop_client_fn stop_client_ptr = NULL;
static start_server_fn start_server_ptr = NULL;
static stop_server_fn stop_server_ptr = NULL;
static get_server_peer_id_fn get_server_peer_id_ptr = NULL;
static free_string_fn free_string_ptr = NULL;

// Handle to the loaded library
static HMODULE lib_handle = NULL;

// Initialize the library
static int initialize_library() {
    if (lib_handle) return 1; // Already initialized
    
    // Load the DLL
    lib_handle = LoadLibraryA("kadugu_net.dll");
    if (!lib_handle) {
        fprintf(stderr, "Failed to load kadugu_net.dll\n");
        return 0;
    }
    
    // Load function pointers
    start_client_ptr = (start_client_fn)GetProcAddress(lib_handle, "start_port_forwarding_client");
    stop_client_ptr = (stop_client_fn)GetProcAddress(lib_handle, "stop_port_forwarding_client");
    start_server_ptr = (start_server_fn)GetProcAddress(lib_handle, "start_port_forwarding_server");
    stop_server_ptr = (stop_server_fn)GetProcAddress(lib_handle, "stop_port_forwarding_server");
    get_server_peer_id_ptr = (get_server_peer_id_fn)GetProcAddress(lib_handle, "get_server_peer_id");
    free_string_ptr = (free_string_fn)GetProcAddress(lib_handle, "free_peer_id");
    
    // Check if all functions were loaded successfully
    if (!start_client_ptr || !stop_client_ptr || !start_server_ptr || 
        !stop_server_ptr || !get_server_peer_id_ptr || !free_string_ptr) {
        fprintf(stderr, "Failed to load one or more functions from kadugu_net.dll\n");
        FreeLibrary(lib_handle);
        lib_handle = NULL;
        return 0;
    }
    
    return 1;
}

// Client functions
KADUGU_NET_API KaduguClientHandle kadugu_start_client(const char* server_peer_id, const char* local_forward_addr) {
    if (!initialize_library()) return NULL;
    return (KaduguClientHandle)start_client_ptr(server_peer_id, local_forward_addr);
}

KADUGU_NET_API void kadugu_stop_client(KaduguClientHandle handle) {
    if (stop_client_ptr && handle) {
        stop_client_ptr(handle);
    }
}

// Server functions
KADUGU_NET_API KaduguServerHandle kadugu_start_server(const char* listen_addr, const char* accepted_peer_id) {
    if (!initialize_library()) return NULL;
    return (KaduguServerHandle)start_server_ptr(listen_addr, accepted_peer_id);
}

KADUGU_NET_API void kadugu_stop_server(KaduguServerHandle handle) {
    if (stop_server_ptr && handle) {
        stop_server_ptr(handle);
    }
}

KADUGU_NET_API char* kadugu_get_server_peer_id(KaduguServerHandle handle) {
    if (get_server_peer_id_ptr && handle) {
        return get_server_peer_id_ptr(handle);
    }
    return NULL;
}

KADUGU_NET_API void kadugu_free_string(char* str) {
    if (free_string_ptr && str) {
        free_string_ptr(str);
    }
}

// Cleanup function
__declspec(dllexport) void kadugu_cleanup() {
    if (lib_handle) {
        FreeLibrary(lib_handle);
        lib_handle = NULL;
    }
}
