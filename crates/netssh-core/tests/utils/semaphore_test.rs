use netssh_core::semaphore::{TimeoutSemaphore, SemaphoreError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[test]
fn test_semaphore_creation() {
    let sem = TimeoutSemaphore::new(5);
    assert_eq!(sem.available_permits().unwrap(), 5);
    assert_eq!(sem.max_permits().unwrap(), 5);
}

#[test]
fn test_semaphore_acquire_release() {
    let sem = TimeoutSemaphore::new(2);
    
    // Acquire first permit
    let permit1 = sem.acquire().unwrap();
    assert_eq!(sem.available_permits().unwrap(), 1);
    
    // Acquire second permit
    let permit2 = sem.acquire().unwrap();
    assert_eq!(sem.available_permits().unwrap(), 0);
    
    // Trying to acquire a third permit should block
    // We'll test this with timeouts in another test
    
    // Release the first permit by dropping it
    drop(permit1);
    assert_eq!(sem.available_permits().unwrap(), 1);
    
    // Release the second permit
    drop(permit2);
    assert_eq!(sem.available_permits().unwrap(), 2);
}

#[test]
fn test_semaphore_try_acquire() {
    let sem = TimeoutSemaphore::new(1);
    
    // Acquire the only permit
    let permit = sem.try_acquire().unwrap();
    assert_eq!(sem.available_permits().unwrap(), 0);
    
    // Trying to acquire another should fail immediately
    assert!(matches!(sem.try_acquire(), Err(SemaphoreError::Timeout)));
    
    // Release the permit
    drop(permit);
    assert_eq!(sem.available_permits().unwrap(), 1);
    
    // Now we should be able to acquire again
    let _ = sem.try_acquire().unwrap();
}

#[test]
fn test_semaphore_timeout() {
    let sem = TimeoutSemaphore::new(1);
    
    // Acquire the only permit
    let permit = sem.acquire().unwrap();
    
    // Try to acquire with a short timeout
    let start = Instant::now();
    let result = sem.acquire_timeout(Some(Duration::from_millis(100)));
    let elapsed = start.elapsed();
    
    // Should timeout and fail
    assert!(matches!(result, Err(SemaphoreError::Timeout)));
    
    // Should have waited close to the timeout period
    assert!(elapsed >= Duration::from_millis(90)); // Allow some timing variance
    assert!(elapsed <= Duration::from_millis(200)); // But not too much
    
    // Release the permit
    drop(permit);
    
    // Now we should be able to acquire with timeout
    let _ = sem.acquire_timeout(Some(Duration::from_millis(100))).unwrap();
}

#[test]
fn test_semaphore_multi_thread() {
    let sem = Arc::new(TimeoutSemaphore::new(3));
    let counter = Arc::new(Mutex::new(0));
    let num_threads = 10;
    let mut handles = Vec::with_capacity(num_threads);
    
    for _ in 0..num_threads {
        let sem_clone = Arc::clone(&sem);
        let counter_clone = Arc::clone(&counter);
        
        let handle = thread::spawn(move || {
            // Acquire a permit
            if let Ok(permit) = sem_clone.acquire_timeout(Some(Duration::from_millis(500))) {
                // Increment the counter
                let mut count = counter_clone.lock().unwrap();
                *count += 1;
                
                // Hold the permit for a short time
                thread::sleep(Duration::from_millis(50));
                
                // Permit will be released when dropped at end of scope
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Check that the counter was incremented,
    // but could be less than num_threads since some may have timed out
    let final_count = *counter.lock().unwrap();
    assert!(final_count > 0);
    assert!(final_count <= num_threads);
    
    // All permits should be back
    assert_eq!(sem.available_permits().unwrap(), 3);
}

#[test]
fn test_semaphore_close() {
    let sem = TimeoutSemaphore::new(5);
    
    // Acquire some permits
    let permit1 = sem.acquire().unwrap();
    let permit2 = sem.acquire().unwrap();
    assert_eq!(sem.available_permits().unwrap(), 3);
    
    // Close the semaphore
    sem.close();
    
    // Try to acquire should fail with Closed error
    assert!(matches!(sem.try_acquire(), Err(SemaphoreError::Closed)));
    assert!(matches!(sem.acquire_timeout(Some(Duration::from_millis(10))), Err(SemaphoreError::Closed)));
    
    // Release the permits - should not affect available count since closed
    drop(permit1);
    drop(permit2);
}

#[test]
fn test_semaphore_add_remove_permits() {
    let sem = TimeoutSemaphore::new(2);
    assert_eq!(sem.max_permits().unwrap(), 2);
    assert_eq!(sem.available_permits().unwrap(), 2);
    
    // Add 3 more permits
    sem.add_permits(3).unwrap();
    assert_eq!(sem.max_permits().unwrap(), 5);
    assert_eq!(sem.available_permits().unwrap(), 5);
    
    // Acquire some permits
    let permit1 = sem.acquire().unwrap();
    let permit2 = sem.acquire().unwrap();
    assert_eq!(sem.available_permits().unwrap(), 3);
    
    // Remove some permits
    sem.remove_permits(2).unwrap();
    assert_eq!(sem.max_permits().unwrap(), 3);
    assert_eq!(sem.available_permits().unwrap(), 1); // Only 1 left because we're still holding 2
    
    // Release held permits
    drop(permit1);
    drop(permit2);
    assert_eq!(sem.available_permits().unwrap(), 3); // All permits available again
}

#[test]
fn test_semaphore_permit_return_on_thread_panic() {
    let sem = Arc::new(TimeoutSemaphore::new(1));
    let sem_clone = Arc::clone(&sem);
    
    assert_eq!(sem.available_permits().unwrap(), 1);
    
    // Spawn a thread that will panic while holding a permit
    let handle = thread::spawn(move || {
        // Acquire the permit
        let _permit = sem_clone.acquire().unwrap();
        assert_eq!(sem_clone.available_permits().unwrap(), 0);
        
        // Panic while holding the permit
        panic!("Deliberate panic for testing");
    });
    
    // The thread should panic, but we just let it complete
    let _ = handle.join();
    
    // The permit should have been released even though the thread panicked
    assert_eq!(sem.available_permits().unwrap(), 1);
} 