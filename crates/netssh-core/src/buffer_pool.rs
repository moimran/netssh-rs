use crate::settings::{get_buffer_setting, BufferSettingType};
use lazy_static::lazy_static;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tracing::{debug, trace};

/// BufferPool provides a mechanism to reuse byte buffers
/// to reduce memory allocations and improve performance.
///
/// This implementation ensures buffers can be safely shared
/// between threads and efficiently reused throughout the
/// application.
pub struct BufferPool {
    pool: Mutex<VecDeque<Vec<u8>>>,
    max_size: usize,
    reuse_threshold: usize,
}

// Global singleton instance of the buffer pool
lazy_static! {
    static ref BUFFER_POOL: Arc<BufferPool> = Arc::new(BufferPool::new(
        get_buffer_setting(BufferSettingType::BufferPoolSize),
        get_buffer_setting(BufferSettingType::BufferReuseThreshold)
    ));
}

impl BufferPool {
    /// Create a new buffer pool with specified max_size
    pub fn new(max_size: usize, reuse_threshold: usize) -> Self {
        debug!(
            "Creating new buffer pool with max_size={}, reuse_threshold={}",
            max_size, reuse_threshold
        );
        Self {
            pool: Mutex::new(VecDeque::with_capacity(max_size)),
            max_size,
            reuse_threshold,
        }
    }

    /// Get the global buffer pool instance
    pub fn global() -> Arc<BufferPool> {
        BUFFER_POOL.clone()
    }

    /// Get a buffer from the pool or create a new one
    pub fn get_buffer(&self, min_capacity: usize) -> BorrowedBuffer {
        let mut pool = self.pool.lock().unwrap();

        // Try to find a buffer with adequate capacity
        let mut buffer = None;
        let mut idx = 0;

        while idx < pool.len() {
            if pool[idx].capacity() >= min_capacity {
                buffer = Some(pool.remove(idx).unwrap());
                break;
            }
            idx += 1;
        }

        // If no adequate buffer was found, create a new one
        let mut vec = buffer.unwrap_or_else(|| {
            trace!("Creating new buffer with capacity {}", min_capacity);
            Vec::with_capacity(min_capacity)
        });

        // Clear the buffer but keep its capacity
        vec.clear();

        BorrowedBuffer {
            buffer: vec,
            pool: Arc::downgrade(&BUFFER_POOL),
            reuse_threshold: self.reuse_threshold,
        }
    }

    /// Return a buffer to the pool if it meets size criteria
    fn return_buffer(&self, mut buffer: Vec<u8>) {
        // Only reuse buffers that are under our threshold to prevent
        // keeping excessively large buffers in memory
        if buffer.capacity() <= self.reuse_threshold {
            buffer.clear();
            let mut pool = self.pool.lock().unwrap();

            // Only add if we're under capacity
            if pool.len() < self.max_size {
                trace!(
                    "Returning buffer with capacity {} to pool",
                    buffer.capacity()
                );
                pool.push_back(buffer);
            }
            // Otherwise let it drop
        }
        // Drop buffers over the threshold
    }
}

/// BorrowedBuffer is a buffer borrowed from the pool
/// It will automatically return to the pool when dropped
pub struct BorrowedBuffer {
    buffer: Vec<u8>,
    pool: std::sync::Weak<BufferPool>,
    #[allow(dead_code)]
    reuse_threshold: usize,
}

impl BorrowedBuffer {
    /// Get a mutable reference to the underlying buffer
    pub fn get_mut(&mut self) -> &mut Vec<u8> {
        &mut self.buffer
    }

    /// Consume the buffer and return the inner Vec<u8>
    /// This prevents returning to the pool on drop
    pub fn into_inner(mut self) -> Vec<u8> {
        std::mem::take(&mut self.buffer)
    }

    /// Read from a reader into this buffer
    pub fn read_from<R: std::io::Read>(
        &mut self,
        reader: &mut R,
        max_len: usize,
    ) -> std::io::Result<usize> {
        // Ensure we have enough capacity
        let current_len = self.buffer.len();
        let needed_capacity = current_len + max_len;

        if self.buffer.capacity() < needed_capacity {
            self.buffer
                .reserve(needed_capacity - self.buffer.capacity());
        }

        // Safety: we just ensured we have enough capacity
        unsafe {
            self.buffer.set_len(current_len + max_len);
        }

        // Read into the buffer
        let read_result = reader.read(&mut self.buffer[current_len..]);

        // Adjust length based on actual bytes read
        if let Ok(bytes_read) = read_result {
            // Safety: truncating is always safe
            unsafe {
                self.buffer.set_len(current_len + bytes_read);
            }
        }

        read_result
    }

    /// Get the buffer as a slice
    pub fn as_slice(&self) -> &[u8] {
        &self.buffer
    }

    /// Convert buffer contents to a string if possible
    pub fn as_utf8_string(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.buffer.clone())
    }

    /// Convert buffer contents to a string, replacing invalid UTF-8 sequences
    pub fn as_utf8_lossy_string(&self) -> String {
        String::from_utf8_lossy(&self.buffer).to_string()
    }

    /// Clear the buffer contents but maintain capacity
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Get the current length of the buffer
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Resize the buffer to the specified length
    pub fn resize(&mut self, new_len: usize, value: u8) {
        self.buffer.resize(new_len, value);
    }
}

impl Drop for BorrowedBuffer {
    fn drop(&mut self) {
        // Only return the buffer to the pool if it's not empty
        if !self.buffer.is_empty() {
            if let Some(pool) = self.pool.upgrade() {
                let buffer = std::mem::take(&mut self.buffer);
                pool.return_buffer(buffer);
            }
        }
    }
}

// Implement common traits for BorrowedBuffer

impl AsRef<[u8]> for BorrowedBuffer {
    fn as_ref(&self) -> &[u8] {
        &self.buffer
    }
}

impl AsMut<[u8]> for BorrowedBuffer {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.buffer
    }
}

impl std::ops::Deref for BorrowedBuffer {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl std::ops::DerefMut for BorrowedBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer
    }
}

impl From<Vec<u8>> for BorrowedBuffer {
    fn from(buffer: Vec<u8>) -> Self {
        let reuse_threshold = get_buffer_setting(BufferSettingType::BufferReuseThreshold);
        Self {
            buffer,
            pool: Arc::downgrade(&BUFFER_POOL),
            reuse_threshold,
        }
    }
}
