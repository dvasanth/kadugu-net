# Kadugu Net

[![Crates.io](https://img.shields.io/crates/v/kadugu-net.svg)](https://crates.io/crates/kadugu-net)
[![Documentation](https://docs.rs/kadugu-net/badge.svg)](https://docs.rs/kadugu-net)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%2FApache--2.0-blue.svg)](https://opensource.org/licenses/MIT)

A high-performance, secure port forwarding library built on top of libp2p, enabling seamless peer-to-peer port forwarding through NATs and firewalls.

## Features

- **Secure Communication**: Built on libp2p's noise protocol for encrypted communication
- **NAT Traversal**: Automatically handles NAT traversal using libp2p's NAT traversal capabilities
- **Async Runtime**: Built with Tokio for high-performance async I/O
- **Cross-platform**: Works on all major platforms
- **FFI Support**: Provides C-compatible FFI for integration with other languages

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
kadugu-net = "0.1.0"
```

## Usage

### Basic Server

```rust
use kadugu_net::PortForwardingServer;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let server = PortForwardingServer::new("0.0.0.0:0", None).await?;
    let peer_id = server.peer_id().to_string();
    println!("Server running with peer ID: {}", peer_id);
    
    // Keep the server running
    tokio::signal::ctrl_c().await?;
    Ok(())
}
```

### Basic Client

```rust
use kadugu_net::PortForwardingClient;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let server_peer_id = "Qm..."; // Replace with actual server peer ID
    let client = PortForwardingClient::new(server_peer_id, "127.0.0.1:8080").await?;
    
    // The client will now forward connections from the server to localhost:8080
    tokio::signal::ctrl_c().await?;
    Ok(())
}
```

## FFI Usage

See the `examples` directory for complete C/C++ examples.

## Building

```bash
# Build the library
cargo build --release

# Build with FFI support
cargo build --release --features="ffi"
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
