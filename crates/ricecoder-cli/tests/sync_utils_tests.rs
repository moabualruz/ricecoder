use ricecoder_cli::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_safe_lock_success() {
        let mutex = Mutex::new(vec![1, 2, 3]);
        let result = safe_lock(&mutex, "test");
        assert!(result.is_ok());
        let guard = result.unwrap();
        assert_eq!(guard.len(), 3);
    }

    #[test]
    fn test_safe_lock_or_default() {
        let mutex: Mutex<Vec<i32>> = Mutex::new(vec![1, 2, 3]);
        let guard = safe_lock_or_default(&mutex, "test");
        assert_eq!(guard.len(), 3);
    }

    #[test]
    fn test_safe_lock_optional_success() {
        let mutex = Mutex::new(vec![1, 2, 3]);
        let result = safe_lock_optional(&mutex, "test");
        assert!(result.is_some());
        let guard = result.unwrap();
        assert_eq!(guard.len(), 3);
    }

    #[test]
    fn test_trait_safe_lock() {
        let mutex = Mutex::new(42);
        let result = mutex.safe_lock("test");
        assert!(result.is_ok());
        assert_eq!(*result.unwrap(), 42);
    }

    #[test]
    fn test_trait_safe_lock_or_default() {
        let mutex: Mutex<Vec<i32>> = Mutex::new(vec![1, 2, 3]);
        let guard = mutex.safe_lock_or_default("test");
        assert_eq!(guard.len(), 3);
    }

    #[test]
    fn test_trait_safe_lock_optional() {
        let mutex = Mutex::new(42);
        let result = mutex.safe_lock_optional("test");
        assert!(result.is_some());
        assert_eq!(*result.unwrap(), 42);
    }

    #[test]
    fn test_concurrent_access() {
        let data = Arc::new(Mutex::new(0));
        let mut handles = vec![];

        for _ in 0..10 {
            let data_clone = Arc::clone(&data);
            let handle = thread::spawn(move || {
                if let Ok(mut guard) = safe_lock(&data_clone, "concurrent test") {
                    *guard += 1;
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let final_value = safe_lock(&data, "final check").unwrap();
        assert_eq!(*final_value, 10);
    }
}
