#ifndef KADUGU_NET_H
#define KADUGU_NET_H

#ifdef _WIN32
    #ifdef KADUGU_NET_EXPORTS
        #define KADUGU_NET_API __declspec(dllexport)
    #else
        #define KADUGU_NET_API __declspec(dllimport)
    #endif
#else
    #define KADUGU_NET_API
#endif

#ifdef __cplusplus
extern "C" {
#endif

typedef void* KaduguClientHandle;
typedef void* KaduguServerHandle;

// Client functions
KADUGU_NET_API KaduguClientHandle kadugu_start_client(const char* server_peer_id, const char* local_forward_addr);
KADUGU_NET_API void kadugu_stop_client(KaduguClientHandle handle);

// Server functions
KADUGU_NET_API KaduguServerHandle kadugu_start_server(const char* listen_addr, const char* accepted_peer_id);
KADUGU_NET_API void kadugu_stop_server(KaduguServerHandle handle);
KADUGU_NET_API char* kadugu_get_server_peer_id(KaduguServerHandle handle);
KADUGU_NET_API void kadugu_free_string(char* str);

#ifdef __cplusplus
}
#endif

#endif // KADUGU_NET_H
