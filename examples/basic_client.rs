//! Basic example of a port forwarding client
//! 
//! This example demonstrates how to create a simple port forwarding client
//! that connects to a server and forwards traffic to a local service.

use kadugu_net::PortForwardingClient;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    // Get server peer ID from command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <server-peer-id>", args[0]);
        std::process::exit(1);
    }
    let server_peer_id = &args[1];

    println!("Starting port forwarding client...");
    println!("Connecting to server with peer ID: {}", server_peer_id);
    
    // Create a new client that will forward connections to localhost:8080
    // Replace "127.0.0.1:8080" with the address of your local service
    let _client = PortForwardingClient::new(server_peer_id, "127.0.0.1:8080").await?;
    
    println!("Client started successfully!");
    println!("Forwarding connections to: 127.0.0.1:8080");
    println!("Press Ctrl+C to stop the client");
    
    // Keep the client running until Ctrl+C is pressed
    tokio::signal::ctrl_c().await?;
    println!("\nShutting down client...");
    
    Ok(())
}
