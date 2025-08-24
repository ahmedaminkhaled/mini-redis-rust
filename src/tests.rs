#[cfg(test)]
mod tests {
    use crate::{new_sharded_db, process, index, N};
    use tokio::net::TcpListener;
    use tokio::time::{sleep, Duration};
    use mini_redis::client;
    use bytes::Bytes;
    use std::net::SocketAddr;

    // Helper function to start a test server
    async fn start_test_server() -> SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        
        // Start server in background
        tokio::spawn(async move {
            let db = new_sharded_db();
            loop {
                let (socket, _) = listener.accept().await.unwrap();
                let db = db.clone();
                tokio::spawn(async move {
                    process(socket, db).await
                });
            }
        });
        
        // Give server time to start
        sleep(Duration::from_millis(10)).await;
        addr
    }

    #[tokio::test]
    async fn test_basic_set_get() {
        let addr = start_test_server().await;
        let mut client = client::connect(addr).await.unwrap();

        // Test SET command
        client.set("hello", "world".into()).await.unwrap();
        
        // Test GET command
        let result = client.get("hello").await.unwrap();
        assert_eq!(result, Some(Bytes::from("world")));
    }

    #[tokio::test]
    async fn test_get_nonexistent_key() {
        let addr = start_test_server().await;
        let mut client = client::connect(addr).await.unwrap();

        // Test GET for non-existent key
        let result = client.get("nonexistent").await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_multiple_clients() {
        let addr = start_test_server().await;
        
        // Create multiple clients
        let mut client1 = client::connect(addr).await.unwrap();
        let mut client2 = client::connect(addr).await.unwrap();

        // Client 1 sets a value
        client1.set("key1", "value1".into()).await.unwrap();
        
        // Client 2 should be able to get the same value
        let result = client2.get("key1").await.unwrap();
        assert_eq!(result, Some(Bytes::from("value1")));

        // Client 2 sets a different value
        client2.set("key2", "value2".into()).await.unwrap();
        
        // Client 1 should be able to get it
        let result = client1.get("key2").await.unwrap();
        assert_eq!(result, Some(Bytes::from("value2")));
    }

    #[tokio::test]
    async fn test_sharding_distribution() {
        let addr = start_test_server().await;
        let mut client = client::connect(addr).await.unwrap();

        // Set multiple keys to test sharding
        let test_keys = vec![
            ("key1", "value1"),
            ("key2", "value2"), 
            ("key3", "value3"),
            ("key4", "value4"),
            ("key5", "value5"),
        ];

        // Set all keys
        for (key, value) in &test_keys {
            client.set(*key, (*value).into()).await.unwrap();
        }

        // Get all keys and verify
        for (key, expected_value) in &test_keys {
            let result = client.get(*key).await.unwrap();
            assert_eq!(result, Some(Bytes::from(*expected_value)));
        }
    }

    #[tokio::test]
    async fn test_overwrite_key() {
        let addr = start_test_server().await;
        let mut client = client::connect(addr).await.unwrap();

        // Set initial value
        client.set("key", "initial".into()).await.unwrap();
        let result = client.get("key").await.unwrap();
        assert_eq!(result, Some(Bytes::from("initial")));

        // Overwrite with new value
        client.set("key", "updated".into()).await.unwrap();
        let result = client.get("key").await.unwrap();
        assert_eq!(result, Some(Bytes::from("updated")));
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        let addr = start_test_server().await;
        
        // Spawn multiple concurrent tasks
        let handles: Vec<_> = (0..10).map(|i| {
            let addr = addr.clone();
            tokio::spawn(async move {
                let mut client = client::connect(addr).await.unwrap();
                let key = format!("concurrent_key_{}", i);
                let value = format!("concurrent_value_{}", i);
                
                client.set(&key, value.clone().into()).await.unwrap();
                let result = client.get(&key).await.unwrap();
                assert_eq!(result, Some(Bytes::from(value)));
            })
        }).collect();

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_hash_distribution() {
        // Test that the hash function distributes keys across shards
        let keys = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"];
        let mut shard_counts = vec![0; N as usize];
        
        for key in &keys {
            let shard = index(key);
            shard_counts[shard] += 1;
        }
        
        // Verify that keys are distributed (not all in one shard)
        let max_count = *shard_counts.iter().max().unwrap();
        let min_count = *shard_counts.iter().min().unwrap();
        
        // Should have some distribution (not all keys in one shard)
        assert!(max_count - min_count <= keys.len());
        assert!(shard_counts.iter().any(|&count| count > 0));
    }

    #[tokio::test]
    async fn test_large_values() {
        let addr = start_test_server().await;
        let mut client = client::connect(addr).await.unwrap();

        // Test with large value (1KB)
        let large_value = "x".repeat(1024);
        client.set("large_key", large_value.clone().into()).await.unwrap();
        
        let result = client.get("large_key").await.unwrap();
        assert_eq!(result, Some(Bytes::from(large_value)));
    }

    #[tokio::test]
    async fn test_empty_value() {
        let addr = start_test_server().await;
        let mut client = client::connect(addr).await.unwrap();

        // Test with empty value
        client.set("empty_key", "".into()).await.unwrap();
        let result = client.get("empty_key").await.unwrap();
        assert_eq!(result, Some(Bytes::from("")));
    }

    #[tokio::test]
    async fn test_special_characters() {
        let addr = start_test_server().await;
        let mut client = client::connect(addr).await.unwrap();

        // Test with special characters
        let special_value = "Hello, 世界! 🦀 ñáéíóú";
        client.set("special_key", special_value.into()).await.unwrap();
        
        let result = client.get("special_key").await.unwrap();
        assert_eq!(result, Some(Bytes::from(special_value)));
    }
}
