# Mini Redis Server

A lightweight, asynchronous Redis-like server implementation written in Rust using Tokio. This project represents my first deep dive into asynchronous programming with Tokio and Redis protocol implementation, showcasing concurrent programming concepts and sharded data storage.

## 🎓 Learning Journey

This project marks an important milestone in my Rust learning path - my **first Tokio-based asynchronous application** and **first Redis implementation**. Through building this server, I've gained hands-on experience with:

### 🔄 **Asynchronous Programming Mastery**
- **Tokio Runtime**: Understanding how async/await works under the hood
- **Task Spawning**: Managing concurrent connections with `tokio::spawn`
- **Async I/O**: Non-blocking network operations and frame processing
- **Future Composition**: Chaining async operations effectively

### 🏗️ **Systems Architecture & Design**
- **Sharding Strategy**: Implementing data partitioning for better performance
- **Concurrency Patterns**: Using `Arc<Mutex<>>` for thread-safe shared state
- **Protocol Implementation**: Understanding and implementing Redis wire protocol
- **Network Programming**: TCP socket handling and connection management

### 🔒 **Rust Ownership & Safety**
- **Smart Pointers**: Practical use of `Arc` for shared ownership
- **Synchronization**: `Mutex` for safe concurrent data access
- **Memory Management**: Efficient handling of `Bytes` and `Vec<u8>`
- **Error Handling**: Working with `Result` types in async contexts

### 🚀 **Key Breakthroughs**
- **From Theory to Practice**: Applying Rust's ownership model in a real concurrent system
- **Performance Considerations**: Understanding why sharding reduces lock contention
- **Protocol Parsing**: Learning how Redis commands are structured and parsed
- **Client-Server Architecture**: Building both server and client components

## 🚀 Features

- **Asynchronous I/O**: Built with Tokio for high-performance async operations
- **Sharded Storage**: Data is distributed across 32 shards for better concurrent access
- **Thread-Safe**: Uses Arc<Mutex<>> for safe concurrent access to shared data
- **Redis Protocol**: Compatible with basic Redis commands (SET, GET)
- **Concurrent Connections**: Handles multiple client connections simultaneously

## 🏗️ Architecture

### Core Components

1. **Main Server**: Listens for TCP connections on `127.0.0.1:6969`
2. **Connection Handler**: Each client connection runs in its own async task
3. **Sharded Database**: Data is partitioned across multiple HashMap shards
4. **Command Processing**: Parses and executes Redis protocol commands

### Sharding Strategy

The server uses a hash-based sharding approach:
- Keys are hashed using Rust's `DefaultHasher`
- Hash value is modulo'd by the number of shards (N=32)
- Each shard is protected by its own mutex for fine-grained locking

```rust
fn index(key: &str) -> usize {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    (hasher.finish() as usize) % (N as usize)
}
```

##  📋 Prerequisites

- Rust 1.70+ 
- Cargo
- Tokio runtime

## 🛠️ Installation

1. Clone the repository:
```bash
git clone <your-repo-url>
cd mini_redis
```

2. Build the project:
```bash
cargo build
```

## 🎯 Usage

### Starting the Server

```bash
cargo run
```

The server will start listening on `127.0.0.1:6969`.

### Running the Client Example

In a separate terminal:

```bash
cargo run --example hello-redis
```

This will run the example client that connects to the server and performs SET/GET operations.

### Interactive Client

The project includes an interactive client where you can input commands in the format `key:value`:

```bash
cargo run --example client-redis
```

Then type commands like:
```
hello:world
foo:bar
test:123
```

## 🔧 Supported Commands

| Command | Description | Example |
|---------|-------------|---------|
| `SET key value` | Store a key-value pair | `SET hello world` |
| `GET key` | Retrieve value for a key | `GET hello` |

## 📝 Code Structure

```
src/
├── main.rs           # Main server implementation
examples/
├── hello-redis.rs    # Basic client example
├── client-redis.rs   # Interactive client
Cargo.toml           # Project dependencies
```

### Key Data Structures

```rust
// Sharded database type
type Shardeddb = Arc<Vec<Mutex<HashMap<String, Vec<u8>>>>>;

// Individual shard structure
type Shard = HashMap<String, Vec<u8>>;
```

## 🔍 Technical Details

### Concurrency Model

- **Server Level**: Uses `tokio::spawn` to handle each connection in a separate task
- **Data Level**: Sharded storage with per-shard mutexes to minimize lock contention
- **Protocol Level**: Async read/write operations using the `mini_redis` connection wrapper

### Memory Management

- Uses `Arc` for shared ownership of database shards
- `Mutex` provides thread-safe access to individual shards
- `Bytes` type for efficient binary data handling
- Cloning is minimized through strategic use of references

### Error Handling

Currently uses `.unwrap()` for simplicity. In production, you'd want to:
- Replace `unwrap()` with proper error propagation
- Handle client disconnections gracefully
- Add logging for debugging and monitoring

## 🧪 Testing

Run the basic functionality test:

```bash
# Terminal 1: Start server
cargo run

# Terminal 2: Run client
cargo run --example client-redis
```

Then type commands in the format `key:value`:
```
hello:world
foo:bar
test:123
```

Expected output:
```
got value from the server; result=world
got value from the server; result=bar
got value from the server; result=123
```

## 🛣️ Future Learning Goals

As this is my first Tokio project, here are the areas I plan to explore next:

### 🔄 **Immediate Improvements**
- [ ] **Error Handling**: Replace `.unwrap()` with proper error handling
- [ ] **Graceful Shutdown**: Implement clean server shutdown with signal handling
- [ ] **Connection Limits**: Add maximum concurrent connection limits
- [ ] **Logging**: Integrate `tracing` for better observability

### 🚀 **Advanced Features**
- [ ] **More Redis Commands**: Implement DEL, EXISTS, EXPIRE, INCR/DECR
- [ ] **Pub/Sub**: Add Redis publish/subscribe functionality
- [ ] **Persistence**: Save data to disk with RDB-style snapshots
- [ ] **Authentication**: Add password-based client authentication
- [ ] **Clustering**: Implement Redis cluster protocol


## 💡 Key Takeaways

Building this Redis server taught me that:

1. **Async is Powerful**: Tokio's async model makes handling thousands of connections surprisingly manageable
2. **Sharding Works**: Distributing data across multiple shards dramatically reduces lock contention
3. **Rust's Safety**: The compiler prevented numerous concurrency bugs that would be common in other languages
4. **Protocol Design Matters**: Understanding how Redis structures its commands gave me insight into network protocol design
5. **Testing is Essential**: The interactive client was crucial for validating the server's behavior

## 🙏 Acknowledgments

- **Tokio Team**: For creating an amazing async runtime and the mini-redis tutorial
- **Rust Community**: For excellent documentation and helpful discussions
- **Redis**: For inspiring this implementation and providing a great protocol reference

## 📚 Learning Resources

**For those starting their own Tokio journey:**
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial) - Official Tokio tutorial (where this project started)
- [mini-redis](https://github.com/tokio-rs/mini-redis) - The reference implementation
- [Async Book](https://rust-lang.github.io/async-book/) - Deep dive into async Rust
- [Redis Protocol Specification](https://redis.io/docs/reference/protocol-spec/) - Understanding the wire protocol

---

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

**Note**: This is an educational project representing my first exploration into Tokio and Redis. While functional, it should not be used in production environments without significant hardening and testing.
