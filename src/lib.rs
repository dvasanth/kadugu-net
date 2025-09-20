//! # Kadugu Net
//! 
//! A high-performance, secure port forwarding library built on top of libp2p.
//! 
//! ## Features
//! 
//! - **Secure Communication**: Built on libp2p's noise protocol for encrypted communication
//! - **NAT Traversal**: Automatically handles NAT traversal using libp2p's NAT traversal capabilities
//! - **Async Runtime**: Built with Tokio for high-performance async I/O
//! - **Cross-platform**: Works on all major platforms
//! - **FFI Support**: Provides C-compatible FFI for integration with other languages
//!
//! ## Example
//! 
//! ```no_run
//! use kadugu_net::{PortForwardingServer, PortForwardingClient};
//! use std::error::Error;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn Error>> {
//!     // Start a server
//!     let server = PortForwardingServer::new("0.0.0.0:0", None).await?;
//!     let server_peer_id = server.peer_id();
//!     
//!     // Start a client that connects to the server
//!     let _client = PortForwardingClient::new(&server_peer_id.to_string(), "127.0.0.1:8080").await?;
//!     
//!     // The client will now forward connections from the server to localhost:8080
//!     tokio::signal::ctrl_c().await?;
//!     Ok(())
//! }
//! ```

mod port_forwarding_client;
mod port_forwarding_server;
mod config;

use std::ffi::{CStr};
use std::net::SocketAddr;
use std::os::raw::c_char;
use std::sync::Mutex;
use std::sync::MutexGuard;

use libp2p::PeerId;
use once_cell::sync::Lazy;
use tokio::runtime::Runtime;

pub use port_forwarding_client::PortForwardingClient;
pub use port_forwarding_server::PortForwardingServer;
pub use config::Config;

/// Opaque handle to a running port forwarding client
/// This is just a type alias for the client pointer for FFI safety
pub type PortForwardingClientHandle = *mut port_forwarding_client::PortForwardingClient;

/// Opaque handle to a running port forwarding server
/// This is just a type alias for the server pointer for FFI safety
pub type PortForwardingServerHandle = *mut port_forwarding_server::PortForwardingServer;

static RUNTIME: Lazy<Mutex<Option<Runtime>>> = Lazy::new(|| Mutex::new(None));

fn get_runtime() -> MutexGuard<'static, Option<Runtime>> {
    let mut runtime = RUNTIME.lock().unwrap();
    if runtime.is_none() {
        *runtime = Some(Runtime::new().unwrap());
    }
    runtime
}


/// Starts the PortForwardingClient
/// 
/// # Safety
/// The returned handle is a raw pointer to a PortForwardingClient that must be freed with stop_port_forwarding_client
#[unsafe(no_mangle)]
pub unsafe extern "C" fn start_port_forwarding_client(
    server_peer: *const c_char,
    local_forward_addr: *const c_char,
) -> PortForwardingClientHandle {
    let server_peer = unsafe { CStr::from_ptr(server_peer).to_string_lossy().to_string() };
    let local_forward_addr = unsafe { CStr::from_ptr(local_forward_addr).to_string_lossy().to_string() };

    let server_peer: PeerId = match server_peer.parse() {
        Ok(p) => p,
        Err(_) => return std::ptr::null_mut(),
    };
    let local_forward_addr: SocketAddr = match local_forward_addr.parse() {
        Ok(a) => a,
        Err(_) => return std::ptr::null_mut(),
    };
    tracing::info!("Connecting using Server peer id {:?} ", server_peer);
    tracing::info!("Connecting using listen addr {:?} ", local_forward_addr);
    let config = config::Config::default();
    let runtime = std::sync::Arc::new(Runtime::new().expect("Failed to create Tokio runtime"));
    let client = Box::new(crate::port_forwarding_client::PortForwardingClient::new(
        server_peer, 
        local_forward_addr, 
        config,
        runtime
    ));
    
    // Get the raw pointer before moving client into the async block
    let client_ptr = Box::into_raw(client);
    
    // Start the client asynchronously
    let client = unsafe { &mut *client_ptr };
    match client.start() {
        Ok(_) => {
            tracing::info!("Client started successfully");
        }
        Err(e) => {
            tracing::error!("Failed to start client: {}", e);
            // Clean up the client on error
            let _ = unsafe { Box::from_raw(client_ptr) };
            return std::ptr::null_mut();
        }
    }
    
    // Create and return the handle with the client pointer
    client_ptr
}

/// Stops the PortForwardingClient and frees its resources
/// 
/// # Safety
/// The handle must be a valid pointer returned by start_port_forwarding_client
#[unsafe(no_mangle)]
pub unsafe extern "C" fn stop_port_forwarding_client(handle: PortForwardingClientHandle) {
    if handle.is_null() {
        return;
    }
    
    // Get the runtime and block on the stop future
    let runtime = get_runtime();
    if let Some(rt) = runtime.as_ref() {
        // Clone the client reference to move into the closure
        let client = unsafe { &mut *handle };
        rt.block_on(async {
            if let Err(e) = client.stop().await {
                tracing::error!("Error stopping client: {}", e);
            }
        });
    }
    
    // Now we can safely drop the client
    unsafe { let _ = Box::from_raw(handle); };
}

/// Starts the PortForwardingServer
/// 
/// # Safety
/// The returned handle is a raw pointer to a PortForwardingServer that must be freed with stop_port_forwarding_server
#[unsafe(no_mangle)]
pub unsafe extern "C" fn start_port_forwarding_server(
    local_server_addr: *const c_char,
    accepted_peer: *const c_char,
) -> PortForwardingServerHandle {
    // Parse listen address
    let server_addr = unsafe { CStr::from_ptr(local_server_addr) }.to_string_lossy().to_string();
    let server_addr: SocketAddr = match server_addr.parse() {
        Ok(a) => a,
        Err(_) => return std::ptr::null_mut(),
    };
    
    // Parse accepted peer (optional)
    let mut accepted_peers = Vec::new();
    if !accepted_peer.is_null() {
        let peer = unsafe { CStr::from_ptr(accepted_peer) }.to_string_lossy().to_string();
        if !peer.is_empty() {
            accepted_peers.push(peer);
        }
    }
    
    let config = config::Config::default();
    let server = Box::new(crate::port_forwarding_server::PortForwardingServer::new(
        server_addr, 
        accepted_peers, 
        config
    ));


    // Get the raw pointer before moving server into the async block
    let server_ptr = Box::into_raw(server);

    // SAFETY: take ownership *again* from raw pointer (once only), inside the task
    let async_server = unsafe { &mut *server_ptr };    
    
    if let Err(e) = async_server.start() {
        tracing::error!("Server error: {:?}", e);
    }

    // Return the server pointer    
    server_ptr
}

/// Stops the PortForwardingServer and frees its resources
/// 
/// # Safety
/// The handle must be a valid pointer returned by start_port_forwarding_server
#[unsafe(no_mangle)]
pub unsafe extern "C" fn stop_port_forwarding_server(server_ptr: PortForwardingServerHandle) {
    if server_ptr.is_null() {
        return;
    }
    
    // Get a reference to the server
    let server = unsafe { &mut *server_ptr };

    tracing::info!("Stopping port forwarding server");
    // Call stop to signal the server to shut down
    let _ = server.stop();
  
    tracing::info!("Port forwarding server stopped");
}

/// Gets the peer ID of the server as a string
/// 
/// # Safety
/// The server_ptr must be a valid pointer to a PortForwardingServer
#[unsafe(no_mangle)]
pub unsafe extern "C" fn get_server_peer_id(server_ptr: PortForwardingServerHandle) -> *const c_char {
    if server_ptr.is_null() {
        return std::ptr::null();
    }
    
    // Get the peer ID from the server
    let peer_id = unsafe {
        let server = &*server_ptr;
        server.peer_id()
    };
    
    // Convert to a C string
    match std::ffi::CString::new(peer_id) {
        Ok(c_string) => {
            // Leak the C string to ensure it lives long enough for the caller
            c_string.into_raw()
        },
        Err(_) => std::ptr::null(),
    }
}

/// Frees a peer ID string returned by get_server_peer_id
/// 
/// # Safety
/// The pointer must be a valid C string previously returned by get_server_peer_id
#[unsafe(no_mangle)]
pub unsafe extern "C" fn free_peer_id(ptr: *mut c_char) {
    if !ptr.is_null() {
        // Reconstruct the CString and let it drop
        unsafe { let _ = std::ffi::CString::from_raw(ptr); }
    }
}

#[cfg(test)]
mod tests {
    use tracing::level_filters::LevelFilter;
    use tracing_subscriber::EnvFilter;

    use super::*;
    use std::ffi::CString;
    use std::time::Duration;

    #[test]
    fn test_port_forwarding_integration() ->  Result<(), Box<dyn std::error::Error>>{
        // Start the server
        let local_server_addr = "192.168.1.101:8080";  // Let the OS choose an available port
        let local_server_addr_cstr = CString::new(local_server_addr).unwrap();

        tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .parse("kadugu")?,
        )
        .init();
            
        // Start the server
        let server_handle = unsafe {
            start_port_forwarding_server(
                local_server_addr_cstr.as_ptr(),
                std::ptr::null(),
            )
        };

        assert!(!server_handle.is_null(), "Server should start successfully");
        
        // Give the server a moment to start
        std::thread::sleep(Duration::from_secs(5));

        // Get the server's peer ID
        let peer_id_ptr = unsafe { get_server_peer_id(server_handle) };
        assert!(!peer_id_ptr.is_null(), "Failed to get server peer ID");
        
        // Convert the C string to a Rust string for verification
        let peer_id_str = unsafe {
            let cstr = CStr::from_ptr(peer_id_ptr);
            let s = cstr.to_str()?.to_string();
            // Free the C string
            free_peer_id(peer_id_ptr as *mut c_char);
            s
        };
        
        tracing::info!("Server peer ID: {}", peer_id_str);
        
        // Start the client
        let local_forward_addr = "127.0.0.1:8081";
        let local_forward_addr_cstr = match CString::new(local_forward_addr) {
            Ok(s) => s,
            Err(_) => {
                unsafe { stop_port_forwarding_server(server_handle); }
                return Err("Failed to create CString for client listen address".into());
            }
        };
        
        let peer_id_cstr = match CString::new(peer_id_str.clone()) {
            Ok(s) => s,
            Err(_) => {
                unsafe { stop_port_forwarding_server(server_handle); }
                return Err("Failed to create CString for peer ID".into());
            }
        };
        
        tracing::info!("Starting client with server peer ID: {}", peer_id_str.clone());
        let client_handle = unsafe {
            start_port_forwarding_client(
                peer_id_cstr.as_ptr(),
                local_forward_addr_cstr.as_ptr(),
            )
        };
        
        if client_handle.is_null() {
            unsafe { stop_port_forwarding_server(server_handle); }
            return Err("Failed to start client".into());
        }
        
        // Let the client and server communicate
        tracing::info!("Client and server started, waiting for communication...");
        std::thread::sleep(Duration::from_secs(10));
        
        // Clean up
        tracing::info!("Stopping client and server...");
        unsafe {
            stop_port_forwarding_client(client_handle);
            std::thread::sleep(Duration::from_secs(1));
            tracing::info!("Client stopped");
            stop_port_forwarding_server(server_handle);
        }
        
        Ok(())
    }
}
