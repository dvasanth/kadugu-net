#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <windows.h>
#include "../kadugu_net.h"

int main(int argc, char* argv[]) {
    if (argc != 3) {
        printf("Usage: %s <server_peer_id> <local_forward_addr>\n", argv[0]);
        printf("Example: %s 12D3KooWExamplePeerId 127.0.0.1:8080\n", argv[0]);
        return 1;
    }

    const char* server_peer_id = argv[1];
    const char* local_forward_addr = argv[2];

    printf("Starting Kadugu client...\n");
    printf("Connecting to server: %s\n", server_peer_id);
    printf("Local forwarding address: %s\n", local_forward_addr);

    // Start the client
    KaduguClientHandle client = kadugu_start_client(server_peer_id, local_forward_addr);
    if (!client) {
        fprintf(stderr, "Failed to start Kadugu client\n");
        return 1;
    }

    printf("Client started successfully. Press Enter to stop...\n");
    getchar();

    // Cleanup
    printf("Stopping client...\n");
    kadugu_stop_client(client);
    
    return 0;
}
