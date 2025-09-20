# C FFI Example

This directory contains an example of how to use the kadugu-net library from C.

## Building the Example

1. First, build the kadugu-net library with FFI support:

```bash
# From the project root
dir
cargo build --release --features=ffi
```

2. Then, compile the C example:

On Linux/macOS:
```bash
gcc -o client main.c -I../.. -L../../target/release -lkadugu_net -lpthread -ldl -lm
```

On Windows:
```cmd
cl /I..\..\include main.c /link /LIBPATH:..\..\target\release\kadugu_net.lib
```

## Running the Example

1. First, start a server using the Rust example:

```bash
cargo run --example basic_server
```

2. In another terminal, run the C client:

```bash
# Replace <server-peer-id> with the peer ID shown by the server
./client <server-peer-id> 127.0.0.1:8080
```

## Notes

- The client will forward connections from the server to the specified local address
- Make sure to replace the peer ID and local address with appropriate values
- The client will run until interrupted with Ctrl+C
