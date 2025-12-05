/// Memory optimization utilities for ricecoder
///
/// This module provides utilities for optimizing memory usage, including:
/// - Memory tracking and profiling
/// - Clone reduction patterns
/// - Streaming utilities for large data
/// - Memory pooling

use std::sync::Arc;
use tracing::{debug, info};

/// Memory usage statistics
#[derive(Debug, Clone, Copy)]
pub struct MemoryStats {
    /// Peak memory usage in bytes
    pub peak_memory: u64,
    /// Current memory usage in bytes
    pub current_memory: u64,
    /// Number of allocations
    pub allocations: u64,
    /// Number of deallocations
    pub deallocations: u64,
}

impl MemoryStats {
    /// Create new memory statistics
    pub fn new() -> Self {
        Self {
            peak_memory: 0,
            current_memory: 0,
            allocations: 0,
            deallocations: 0,
        }
    }

    /// Update peak memory
    pub fn update_peak(&mut self, current: u64) {
        if current > self.peak_memory {
            self.peak_memory = current;
        }
    }

    /// Format memory size as human-readable string
    pub fn format_size(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
        let mut size = bytes as f64;
        let mut unit_idx = 0;

        while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
            size /= 1024.0;
            unit_idx += 1;
        }

        format!("{:.2} {}", size, UNITS[unit_idx])
    }

    /// Format statistics as string
    pub fn format(&self) -> String {
        format!(
            "Memory: peak={}, current={}, allocations={}, deallocations={}",
            Self::format_size(self.peak_memory),
            Self::format_size(self.current_memory),
            self.allocations,
            self.deallocations
        )
    }
}

impl Default for MemoryStats {
    fn default() -> Self {
        Self::new()
    }
}

/// String interning for reducing duplicate strings
pub struct StringIntern {
    strings: std::sync::Mutex<std::collections::HashMap<String, Arc<str>>>,
}

impl StringIntern {
    /// Create new string intern
    pub fn new() -> Self {
        Self {
            strings: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }

    /// Intern a string (returns shared reference)
    pub fn intern(&self, s: &str) -> Arc<str> {
        let mut map = self.strings.lock().unwrap();
        
        if let Some(interned) = map.get(s) {
            Arc::clone(interned)
        } else {
            let arc: Arc<str> = Arc::from(s);
            map.insert(s.to_string(), Arc::clone(&arc));
            arc
        }
    }

    /// Get statistics
    pub fn stats(&self) -> usize {
        self.strings.lock().unwrap().len()
    }

    /// Clear all interned strings
    pub fn clear(&self) {
        self.strings.lock().unwrap().clear();
    }
}

impl Default for StringIntern {
    fn default() -> Self {
        Self::new()
    }
}

/// Object pool for reusing temporary objects
pub struct ObjectPool<T> {
    pool: std::sync::Mutex<Vec<T>>,
    factory: Box<dyn Fn() -> T + Send + Sync>,
}

impl<T: Send + Sync + 'static> ObjectPool<T> {
    /// Create new object pool
    pub fn new<F: Fn() -> T + Send + Sync + 'static>(factory: F) -> Self {
        Self {
            pool: std::sync::Mutex::new(Vec::new()),
            factory: Box::new(factory),
        }
    }

    /// Get object from pool or create new one
    pub fn get(&self) -> T {
        let mut pool = self.pool.lock().unwrap();
        pool.pop().unwrap_or_else(|| (self.factory)())
    }

    /// Return object to pool
    pub fn return_object(&self, obj: T) {
        let mut pool = self.pool.lock().unwrap();
        if pool.len() < 100 {  // Limit pool size
            pool.push(obj);
        }
    }

    /// Get pool size
    pub fn size(&self) -> usize {
        self.pool.lock().unwrap().len()
    }

    /// Clear pool
    pub fn clear(&self) {
        self.pool.lock().unwrap().clear();
    }
}

/// Streaming buffer for processing large data
pub struct StreamingBuffer {
    buffer: Vec<u8>,
    capacity: usize,
}

impl StreamingBuffer {
    /// Create new streaming buffer with given capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            capacity,
        }
    }

    /// Get mutable reference to buffer
    pub fn as_mut(&mut self) -> &mut Vec<u8> {
        &mut self.buffer
    }

    /// Get reference to buffer
    pub fn as_ref(&self) -> &[u8] {
        &self.buffer
    }

    /// Clear buffer for reuse
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Get current size
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Get capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

impl Default for StreamingBuffer {
    fn default() -> Self {
        Self::new(8192)  // 8KB default
    }
}

/// Memory optimization recommendations
pub struct MemoryOptimizationReport {
    /// Identified optimization opportunities
    pub opportunities: Vec<String>,
    /// Estimated memory savings
    pub estimated_savings: u64,
    /// Priority level (1-5, 1 being highest)
    pub priority: u32,
}

impl MemoryOptimizationReport {
    /// Create new report
    pub fn new() -> Self {
        Self {
            opportunities: Vec::new(),
            estimated_savings: 0,
            priority: 3,
        }
    }

    /// Add optimization opportunity
    pub fn add_opportunity(&mut self, opportunity: String, savings: u64) {
        self.opportunities.push(opportunity);
        self.estimated_savings += savings;
    }

    /// Format report as string
    pub fn format(&self) -> String {
        let mut result = format!(
            "Memory Optimization Report\n\
             Estimated Savings: {}\n\
             Priority: {}/5\n\
             Opportunities:\n",
            MemoryStats::format_size(self.estimated_savings),
            self.priority
        );

        for (i, opp) in self.opportunities.iter().enumerate() {
            result.push_str(&format!("  {}. {}\n", i + 1, opp));
        }

        result
    }

    /// Log report
    pub fn log(&self) {
        info!("{}", self.format());
    }
}

impl Default for MemoryOptimizationReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory optimization patterns
pub mod patterns {
    use super::*;

    /// Pattern 1: Use references instead of clones
    /// 
    /// Before:
    /// ```ignore
    /// let config_copy = config.clone();
    /// process(&config_copy);
    /// ```
    /// 
    /// After:
    /// ```ignore
    /// process(&config);
    /// ```
    pub fn reduce_clones() -> &'static str {
        "Use references instead of cloning large data structures"
    }

    /// Pattern 2: Use Arc for shared ownership
    /// 
    /// Before:
    /// ```ignore
    /// let data = vec![1, 2, 3];
    /// let data1 = data.clone();
    /// let data2 = data.clone();
    /// ```
    /// 
    /// After:
    /// ```ignore
    /// let data = Arc::new(vec![1, 2, 3]);
    /// let data1 = Arc::clone(&data);
    /// let data2 = Arc::clone(&data);
    /// ```
    pub fn use_arc_for_sharing() -> &'static str {
        "Use Arc for shared ownership instead of cloning"
    }

    /// Pattern 3: Stream large files instead of loading into memory
    /// 
    /// Before:
    /// ```ignore
    /// let content = std::fs::read_to_string(path)?;
    /// for line in content.lines() { ... }
    /// ```
    /// 
    /// After:
    /// ```ignore
    /// let file = std::fs::File::open(path)?;
    /// let reader = std::io::BufReader::new(file);
    /// for line in reader.lines() { ... }
    /// ```
    pub fn stream_large_files() -> &'static str {
        "Stream large files instead of loading into memory"
    }

    /// Pattern 4: Use string interning for duplicate strings
    /// 
    /// Before:
    /// ```ignore
    /// let s1 = "provider".to_string();
    /// let s2 = "provider".to_string();
    /// ```
    /// 
    /// After:
    /// ```ignore
    /// let intern = StringIntern::new();
    /// let s1 = intern.intern("provider");
    /// let s2 = intern.intern("provider");
    /// ```
    pub fn use_string_interning() -> &'static str {
        "Use string interning for duplicate strings"
    }

    /// Pattern 5: Use object pooling for temporary objects
    /// 
    /// Before:
    /// ```ignore
    /// for item in items {
    ///     let buffer = Vec::new();
    ///     process(item, buffer);
    /// }
    /// ```
    /// 
    /// After:
    /// ```ignore
    /// let pool = ObjectPool::new(Vec::new);
    /// for item in items {
    ///     let mut buffer = pool.get();
    ///     process(item, &mut buffer);
    ///     pool.return_object(buffer);
    /// }
    /// ```
    pub fn use_object_pooling() -> &'static str {
        "Use object pooling for temporary objects"
    }

    /// Pattern 6: Use compact data structures
    /// 
    /// Before:
    /// ```ignore
    /// struct FileInfo {
    ///     path: String,
    ///     size: u64,
    ///     modified: DateTime,
    /// }
    /// ```
    /// 
    /// After:
    /// ```ignore
    /// struct FileInfo {
    ///     path: Arc<str>,
    ///     size: u32,
    ///     modified: u64,
    /// }
    /// ```
    pub fn use_compact_structures() -> &'static str {
        "Use compact data structures with smaller types"
    }

    /// Pattern 7: Use lazy evaluation
    /// 
    /// Before:
    /// ```ignore
    /// let files: Vec<_> = read_dir(path)?
    ///     .map(|e| analyze_file(&e.path()?))
    ///     .collect()?;
    /// ```
    /// 
    /// After:
    /// ```ignore
    /// let files = read_dir(path)?
    ///     .filter_map(|e| analyze_file(&e.path()).ok());
    /// ```
    pub fn use_lazy_evaluation() -> &'static str {
        "Use lazy evaluation with iterators"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_stats_format_size() {
        assert_eq!(MemoryStats::format_size(512), "512.00 B");
        assert_eq!(MemoryStats::format_size(1024), "1.00 KB");
        assert_eq!(MemoryStats::format_size(1024 * 1024), "1.00 MB");
    }

    #[test]
    fn test_string_intern() {
        let intern = StringIntern::new();
        
        let s1 = intern.intern("test");
        let s2 = intern.intern("test");
        
        // Both should point to same data
        assert_eq!(s1.as_ptr(), s2.as_ptr());
        assert_eq!(intern.stats(), 1);
    }

    #[test]
    fn test_object_pool() {
        let pool = ObjectPool::new(Vec::<u8>::new);
        
        let mut obj1 = pool.get();
        obj1.push(1);
        
        pool.return_object(obj1);
        
        let obj2 = pool.get();
        assert_eq!(obj2.len(), 1);
        assert_eq!(pool.size(), 0);
    }

    #[test]
    fn test_streaming_buffer() {
        let mut buffer = StreamingBuffer::new(1024);
        
        assert!(buffer.is_empty());
        assert_eq!(buffer.capacity(), 1024);
        
        buffer.as_mut().push(1);
        assert_eq!(buffer.len(), 1);
        
        buffer.clear();
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_memory_optimization_report() {
        let mut report = MemoryOptimizationReport::new();
        
        report.add_opportunity("Reduce clones".to_string(), 1024 * 1024);
        report.add_opportunity("Stream files".to_string(), 512 * 1024);
        
        assert_eq!(report.opportunities.len(), 2);
        assert_eq!(report.estimated_savings, 1536 * 1024);
    }
}
