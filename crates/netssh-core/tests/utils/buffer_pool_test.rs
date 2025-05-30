use netssh_core::buffer_pool::BufferPool;
use std::sync::Arc;

#[test]
fn test_buffer_pool_creation() {
    let pool = BufferPool::new(10, 8192);
    assert!(pool.get_buffer(1024).len() == 0);
}

#[test]
fn test_buffer_pool_reuse() {
    let pool = Arc::new(BufferPool::new(5, 8192));

    // Get and release multiple buffers
    for _ in 0..10 {
        let mut buffer = pool.get_buffer(1024);
        buffer.resize(100, 42);

        // Let buffer drop and return to pool
    }

    // Get another buffer, should be from the pool
    let mut buffer = pool.get_buffer(1024);

    // Buffer should be empty even though we previously filled one
    assert_eq!(buffer.len(), 0);

    // But it should have capacity
    assert!(buffer.get_mut().capacity() >= 1024);
}

#[test]
fn test_buffer_pool_capacity() {
    let pool = Arc::new(BufferPool::new(3, 8192));

    // Create more buffers than the pool capacity
    let mut buffers = Vec::new();
    for i in 0..5 {
        let mut buffer = pool.get_buffer(1024 * (i + 1));
        buffer.resize(100, 42);
        buffers.push(buffer);
    }

    // Drop all buffers
    buffers.clear();

    // The pool should now have 3 buffers (its max capacity)
    let buffer = pool.get_buffer(1024);
    assert_eq!(buffer.len(), 0);
}

#[test]
fn test_borrowed_buffer_operations() {
    let pool = Arc::new(BufferPool::new(5, 8192));
    let mut buffer = pool.get_buffer(1024);

    // Test resizing
    buffer.resize(100, 42);
    assert_eq!(buffer.len(), 100);
    assert!(buffer.get_mut().capacity() >= 100);

    // Test clear
    buffer.clear();
    assert_eq!(buffer.len(), 0);

    // Test is_empty
    assert!(buffer.is_empty());

    // Test as_slice and indexing
    buffer.resize(10, 1);
    assert_eq!(buffer.as_slice().len(), 10);
    assert_eq!(buffer[0], 1);

    // Test into_inner
    let inner = buffer.into_inner();
    assert_eq!(inner.len(), 10);
}

#[test]
fn test_borrowed_buffer_read_from() {
    let pool = Arc::new(BufferPool::new(5, 8192));
    let mut buffer = pool.get_buffer(1024);

    // Create a test data source
    let data = [1, 2, 3, 4, 5];
    let mut cursor = std::io::Cursor::new(data);

    // Read from the cursor
    let bytes_read = buffer.read_from(&mut cursor, 5).unwrap();
    assert_eq!(bytes_read, 5);
    assert_eq!(buffer.len(), 5);
    assert_eq!(&buffer[0..5], &[1, 2, 3, 4, 5]);

    // Read more data (should be at EOF)
    let bytes_read = buffer.read_from(&mut cursor, 5).unwrap();
    assert_eq!(bytes_read, 0);
    assert_eq!(buffer.len(), 5); // Length shouldn't change
}

#[test]
fn test_global_buffer_pool() {
    // Test the global instance
    let pool = BufferPool::global();

    // Get a buffer from the global pool
    let mut buffer = pool.get_buffer(4096);
    buffer.resize(100, 255);
    assert_eq!(buffer.len(), 100);

    // Drop the buffer and it should return to the pool
    drop(buffer);

    // Get another buffer, should be from the pool
    let buffer = pool.get_buffer(2048);
    assert_eq!(buffer.len(), 0); // Should be empty
}

#[test]
fn test_buffer_utf8_conversion() {
    let pool = Arc::new(BufferPool::new(5, 8192));
    let mut buffer = pool.get_buffer(1024);

    // Valid UTF-8
    buffer.clear();
    buffer
        .get_mut()
        .extend_from_slice("Hello, world!".as_bytes());
    assert_eq!(buffer.as_utf8_string().unwrap(), "Hello, world!");

    // Invalid UTF-8
    buffer.clear();
    buffer
        .get_mut()
        .extend_from_slice(&[0xFF, 0x00, 0x80, 0xBF]);
    assert!(buffer.as_utf8_string().is_err());

    // Test lossy conversion
    assert!(buffer.as_utf8_lossy_string().len() > 0);
}

#[test]
fn test_buffer_reuse_threshold() {
    let pool = Arc::new(BufferPool::new(5, 1024)); // Set reuse threshold to 1024

    // Create a buffer larger than the threshold
    let mut large_buffer = pool.get_buffer(2048);
    large_buffer.resize(1500, 1);
    drop(large_buffer);

    // Create a buffer smaller than the threshold
    let mut small_buffer = pool.get_buffer(512);
    small_buffer.resize(100, 1);
    drop(small_buffer);

    // Now get a buffer - the small one should have been returned to the pool
    let mut buffer = pool.get_buffer(256);

    // If this capacity matches the small buffer's capacity, it was reused
    assert!(buffer.get_mut().capacity() >= 256);
    assert!(buffer.get_mut().capacity() < 2048); // Should not be the large buffer
}
