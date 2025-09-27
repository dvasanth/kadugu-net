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

### End-to-end example (matches the unit test)

This example starts a server, retrieves its peer ID, starts a client targeting that peer, waits for some time, and then stops both. It uses the crate's exported FFI-style functions (which are also callable from Rust) as used in the unit test.

```rust
use std::ffi::{CStr, CString};
use std::time::Duration;

use kadugu_net::{
    free_peer_id,
    get_server_peer_id,
    start_port_forwarding_client,
    start_port_forwarding_server,
    stop_port_forwarding_client,
    stop_port_forwarding_server,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Start the server listening on a local address
    let server_addr = CString::new("127.0.0.1:8081")?;
    // No accepted peer filter (accept all)
    let accepted_peer: *const std::os::raw::c_char = std::ptr::null();

    let server_handle = unsafe { start_port_forwarding_server(server_addr.as_ptr(), accepted_peer) };
    if server_handle.is_null() {
        return Err("Failed to start server".into());
    }

    // Get the server peer id as a string
    let peer_cstr = unsafe { get_server_peer_id(server_handle) };
    if peer_cstr.is_null() {
        unsafe { stop_port_forwarding_server(server_handle) };
        return Err("Failed to get server peer id".into());
    }
    let server_peer_id = unsafe { CStr::from_ptr(peer_cstr) }.to_string_lossy().to_string();
    // Free the allocated C string
    unsafe { free_peer_id(peer_cstr as *mut _) };

    println!("Server peer ID: {}", server_peer_id);

    // Start the client: forward server connections to a local target (e.g., 127.0.0.1:8080)
    let peer_c = CString::new(server_peer_id)?;
    let local_target = CString::new("127.0.0.1:8080")?;
    let client_handle = unsafe { start_port_forwarding_client(peer_c.as_ptr(), local_target.as_ptr()) };
    if client_handle.is_null() {
        unsafe { stop_port_forwarding_server(server_handle) };
        return Err("Failed to start client".into());
    }

    // Let the client and server communicate
    println!("Client and server started, waiting for communication...");
    std::thread::sleep(Duration::from_secs(30));

    // Clean up
    println!("Stopping client and server...");
    unsafe {
        stop_port_forwarding_client(client_handle);
        std::thread::sleep(Duration::from_secs(1));
        stop_port_forwarding_server(server_handle);
    }

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
