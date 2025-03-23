use std::sync::{Arc, Mutex, Condvar};
use std::time::{Duration, Instant};
use log::{debug, trace, warn};
use thiserror::Error;

/// Error types for semaphore operations
#[derive(Error, Debug)]
pub enum SemaphoreError {
    #[error("Timed out waiting for semaphore permit")]
    Timeout,
    
    #[error("Semaphore is closed")]
    Closed,
    
    #[error("Failed to acquire lock: {0}")]
    LockError(String),
}

/// A counting semaphore implementation with timeout support
/// 
/// This semaphore allows a maximum number of permits to be acquired
/// and supports timeout-based acquisition to prevent excessive queuing
/// during high loads.
pub struct TimeoutSemaphore {
    state: Arc<(Mutex<SemaphoreState>, Condvar)>,
}

/// Internal state for the semaphore
struct SemaphoreState {
    /// Current available permits
    available: usize,
    
    /// Maximum number of permits
    max_permits: usize,
    
    /// Whether the semaphore is closed
    closed: bool,
}

/// A permit acquired from the semaphore
/// When dropped, the permit is automatically returned to the semaphore
pub struct SemaphorePermit {
    semaphore: Arc<(Mutex<SemaphoreState>, Condvar)>,
}

impl Drop for SemaphorePermit {
    fn drop(&mut self) {
        let (lock, cvar) = &*self.semaphore;
        if let Ok(mut state) = lock.lock() {
            if !state.closed {
                state.available += 1;
                trace!("Permit released, available: {}/{}", state.available, state.max_permits);
                cvar.notify_one();
            }
        }
    }
}

impl TimeoutSemaphore {
    /// Create a new semaphore with the specified max_permits
    pub fn new(max_permits: usize) -> Self {
        debug!("Creating new semaphore with max_permits={}", max_permits);
        let state = SemaphoreState {
            available: max_permits,
            max_permits,
            closed: false,
        };
        
        Self {
            state: Arc::new((Mutex::new(state), Condvar::new())),
        }
    }
    
    /// Try to acquire a permit without waiting
    pub fn try_acquire(&self) -> Result<SemaphorePermit, SemaphoreError> {
        let (lock, _) = &*self.state;
        let mut state = lock.lock()
            .map_err(|e| SemaphoreError::LockError(e.to_string()))?;
        
        if state.closed {
            return Err(SemaphoreError::Closed);
        }
        
        if state.available > 0 {
            state.available -= 1;
            trace!("Permit acquired immediately, remaining: {}/{}", state.available, state.max_permits);
            Ok(SemaphorePermit {
                semaphore: self.state.clone(),
            })
        } else {
            Err(SemaphoreError::Timeout)
        }
    }
    
    /// Acquire a permit, waiting indefinitely if necessary
    pub fn acquire(&self) -> Result<SemaphorePermit, SemaphoreError> {
        self.acquire_timeout(None)
    }
    
    /// Acquire a permit with a timeout
    pub fn acquire_timeout(&self, timeout: Option<Duration>) -> Result<SemaphorePermit, SemaphoreError> {
        let (lock, cvar) = &*self.state;
        let mut state = lock.lock()
            .map_err(|e| SemaphoreError::LockError(e.to_string()))?;
        
        if state.closed {
            return Err(SemaphoreError::Closed);
        }
        
        // Fast path: if a permit is available immediately, take it
        if state.available > 0 {
            state.available -= 1;
            trace!("Permit acquired immediately, remaining: {}/{}", state.available, state.max_permits);
            return Ok(SemaphorePermit {
                semaphore: self.state.clone(),
            });
        }
        
        // If no timeout, wait indefinitely
        if timeout.is_none() {
            debug!("Waiting for permit (indefinitely)");
            state = cvar.wait_while(state, |s| s.available == 0 && !s.closed)
                .map_err(|e| SemaphoreError::LockError(e.to_string()))?;
                
            if state.closed {
                return Err(SemaphoreError::Closed);
            }
            
            state.available -= 1;
            trace!("Permit acquired after waiting, remaining: {}/{}", state.available, state.max_permits);
            return Ok(SemaphorePermit {
                semaphore: self.state.clone(),
            });
        }
        
        // Wait with timeout
        let timeout = timeout.unwrap();
        let start = Instant::now();
        debug!("Waiting for permit with timeout: {:?}", timeout);
        
        loop {
            let elapsed = start.elapsed();
            if elapsed >= timeout {
                warn!("Timeout waiting for semaphore permit");
                return Err(SemaphoreError::Timeout);
            }
            
            let remaining = timeout - elapsed;
            let result = cvar.wait_timeout_while(state, remaining, |s| s.available == 0 && !s.closed)
                .map_err(|e| SemaphoreError::LockError(e.to_string()))?;
                
            state = result.0;
            let timed_out = result.1.timed_out();
            
            if state.closed {
                return Err(SemaphoreError::Closed);
            }
            
            if !timed_out && state.available > 0 {
                state.available -= 1;
                trace!("Permit acquired after waiting, remaining: {}/{}", state.available, state.max_permits);
                return Ok(SemaphorePermit {
                    semaphore: self.state.clone(),
                });
            }
            
            if timed_out {
                warn!("Timeout waiting for semaphore permit");
                return Err(SemaphoreError::Timeout);
            }
        }
    }
    
    /// Get the current number of available permits
    pub fn available_permits(&self) -> Result<usize, SemaphoreError> {
        let (lock, _) = &*self.state;
        let state = lock.lock()
            .map_err(|e| SemaphoreError::LockError(e.to_string()))?;
        
        Ok(state.available)
    }
    
    /// Get the maximum number of permits
    pub fn max_permits(&self) -> Result<usize, SemaphoreError> {
        let (lock, _) = &*self.state;
        let state = lock.lock()
            .map_err(|e| SemaphoreError::LockError(e.to_string()))?;
        
        Ok(state.max_permits)
    }
    
    /// Close the semaphore, preventing further acquisitions
    pub fn close(&self) {
        let (lock, cvar) = &*self.state;
        if let Ok(mut state) = lock.lock() {
            state.closed = true;
            cvar.notify_all();
        }
    }
    
    /// Add permits to the semaphore
    pub fn add_permits(&self, count: usize) -> Result<(), SemaphoreError> {
        let (lock, cvar) = &*self.state;
        let mut state = lock.lock()
            .map_err(|e| SemaphoreError::LockError(e.to_string()))?;
        
        if state.closed {
            return Err(SemaphoreError::Closed);
        }
        
        state.max_permits += count;
        state.available += count;
        
        debug!("Added {} permits, now available: {}/{}", 
               count, state.available, state.max_permits);
        
        cvar.notify_all();
        Ok(())
    }
    
    /// Remove permits from the semaphore
    /// This will not affect permits that have already been acquired
    pub fn remove_permits(&self, count: usize) -> Result<(), SemaphoreError> {
        let (lock, _) = &*self.state;
        let mut state = lock.lock()
            .map_err(|e| SemaphoreError::LockError(e.to_string()))?;
        
        if state.closed {
            return Err(SemaphoreError::Closed);
        }
        
        let remove = count.min(state.max_permits);
        state.max_permits -= remove;
        state.available = state.available.min(state.max_permits);
        
        debug!("Removed {} permits, now available: {}/{}", 
               remove, state.available, state.max_permits);
        
        Ok(())
    }
} 