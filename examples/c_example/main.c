#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

// Include the generated header
#include <kadugu_net.h>

int main(int argc, char *argv[]) {
    if (argc != 3) {
        printf("Usage: %s <server_peer_id> <local_forward_addr>\n", argv[0]);
        return 1;
    }

    const char* server_peer = argv[1];
    const char* local_forward_addr = argv[2];

    printf("Starting port forwarding client...\n");
    printf("Server Peer ID: %s\n", server_peer);
    printf("Local Forward Address: %s\n", local_forward_addr);

    // Start the client
    PortForwardingClientHandle client = start_port_forwarding_client(
        server_peer,
        local_forward_addr
    );

    if (client == NULL) {
        printf("Failed to start client\n");
        return 1;
    }

    printf("Client started successfully!\n");
    printf("Press Ctrl+C to stop\n");

    // Keep the client running until interrupted
    while (1) {
        sleep(1);
    }

    // Clean up (this won't be reached in this example)
    stop_port_forwarding_client(client);
    return 0;
}
