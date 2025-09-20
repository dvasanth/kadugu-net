//! Basic example of a port forwarding server
//!
//! This example demonstrates how to create a simple port forwarding server
//! that can accept connections from clients and forward them to a local service.

use kadugu_net::PortForwardingServer;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    tracing_subscriber::fmt().with_env_filter("info").init();

    println!("Starting port forwarding server...");

    // Create a new server on a random available port
    let server = PortForwardingServer::new("0.0.0.0:8080", None).await?;

    // Get the server's peer ID and listening address
    let peer_id = server.peer_id();
    let listen_addr = server.listen_addr();

    println!("Server started successfully!");
    println!("Peer ID: {}", peer_id);
    println!("Listening on: {}", listen_addr);
    println!("\nYou can now start a client with this peer ID to establish a connection.");

    // Keep the server running until Ctrl+C is pressed
    tokio::signal::ctrl_c().await?;
    println!("\nShutting down server...");
    Ok(())
}
