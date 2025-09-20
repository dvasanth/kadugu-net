# Kadugu Net C Bindings

C bindings for the Kadugu Net port forwarding library.

## Prerequisites
- Windows with MSVC
- CMake 3.10+
- Rust toolchain

## Building
1. Build Rust library:
   ```bash
   cargo build --release
   ```
2. Build C examples:
   ```bash
   mkdir build
   cd build
   cmake .. -G "Visual Studio 16 2019" -A x64
   cmake --build . --config Release
   ```

## Usage
### Server
```bash
kadugu_server_example.exe 0.0.0.0:4001 PEER_ID
```

### Client
```bash
kadugu_client_example.exe SERVER_PEER_ID 127.0.0.1:8080
```

## API
- `KaduguClientHandle kadugu_start_client(server_peer_id, local_forward_addr)`
- `void kadugu_stop_client(handle)`
- `KaduguServerHandle kadugu_start_server(listen_addr, accepted_peer_id)`
- `void kadugu_stop_server(handle)`
- `char* kadugu_get_server_peer_id(handle)`
- `void kadugu_free_string(str)`

## Notes
- Ensure `kadugu_net.dll` is in the same directory as your executable.
- The library is not thread-safe.
