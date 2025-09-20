#include <stdio.h>
#include <stdlib.h>
#include <windows.h>
#include "../kadugu_net.h"

int main(int argc, char* argv[]) {
    if (argc != 3) {
        printf("Usage: %s <listen_addr> <accepted_peer_id>\n", argv[0]);
        printf("Example: %s 0.0.0.0:4001 12D3KooWExamplePeerId\n", argv[0]);
        return 1;
    }

    const char* listen_addr = argv[1];
    const char* accepted_peer_id = argv[2];

    printf("Starting Kadugu server...\n");
    printf("Listening on: %s\n", listen_addr);
    printf("Accepting peer: %s\n", accepted_peer_id);

    // Start the server
    KaduguServerHandle server = kadugu_start_server(listen_addr, accepted_peer_id);
    if (!server) {
        fprintf(stderr, "Failed to start Kadugu server\n");
        return 1;
    }

    // Get and print the server's peer ID
    char* peer_id = kadugu_get_server_peer_id(server);
    if (peer_id) {
        printf("Server Peer ID: %s\n", peer_id);
        printf("Share this ID with clients to allow them to connect.\n");
        kadugu_free_string(peer_id);
    }

    printf("Server started successfully. Press Enter to stop...\n");
    getchar();

    // Cleanup
    printf("Stopping server...\n");
    kadugu_stop_server(server);
    
    return 0;
}
