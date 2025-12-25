//! Cache entry compression
//!
//! Provides compression utilities for cache entries to reduce storage size.
//! Uses gzip compression via flate2.

use std::io::{Read, Write};

use flate2::{read::GzDecoder, write::GzEncoder, Compression};

use crate::error::{CacheError, Result};

/// Compression level for cache entries
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionLevel {
    /// No compression (passthrough)
    None,
    /// Fast compression with lower ratio
    Fast,
    /// Balanced compression (default)
    Default,
    /// Best compression ratio (slower)
    Best,
}

impl CompressionLevel {
    /// Convert to flate2 Compression level
    fn to_flate2(&self) -> Option<Compression> {
        match self {
            CompressionLevel::None => None,
            CompressionLevel::Fast => Some(Compression::fast()),
            CompressionLevel::Default => Some(Compression::default()),
            CompressionLevel::Best => Some(Compression::best()),
        }
    }
}

impl Default for CompressionLevel {
    fn default() -> Self {
        CompressionLevel::Default
    }
}

/// Compressor for cache entries
#[derive(Debug, Clone)]
pub struct CacheCompressor {
    /// Compression level to use
    level: CompressionLevel,
    /// Minimum size in bytes before compression is applied
    min_size: usize,
}

impl CacheCompressor {
    /// Create a new compressor with default settings
    pub fn new() -> Self {
        Self {
            level: CompressionLevel::Default,
            min_size: 100, // Don't compress entries smaller than 100 bytes
        }
    }

    /// Create a compressor with specific level
    pub fn with_level(level: CompressionLevel) -> Self {
        Self {
            level,
            min_size: 100,
        }
    }

    /// Set minimum size for compression
    ///
    /// Entries smaller than this size will not be compressed.
    pub fn with_min_size(mut self, min_size: usize) -> Self {
        self.min_size = min_size;
        self
    }

    /// Compress data if it meets the size threshold
    ///
    /// Returns the compressed data and a flag indicating if compression was applied.
    pub fn compress(&self, data: &[u8]) -> Result<CompressedData> {
        // Skip compression for small entries or if compression is disabled
        if data.len() < self.min_size || self.level == CompressionLevel::None {
            return Ok(CompressedData {
                data: data.to_vec(),
                is_compressed: false,
                original_size: data.len(),
            });
        }

        let compression = match self.level.to_flate2() {
            Some(c) => c,
            None => {
                return Ok(CompressedData {
                    data: data.to_vec(),
                    is_compressed: false,
                    original_size: data.len(),
                });
            }
        };

        let mut encoder = GzEncoder::new(Vec::new(), compression);
        encoder.write_all(data).map_err(|e| CacheError::Compression {
            message: format!("Failed to compress data: {}", e),
        })?;

        let compressed = encoder.finish().map_err(|e| CacheError::Compression {
            message: format!("Failed to finalize compression: {}", e),
        })?;

        // Only use compressed version if it's actually smaller
        if compressed.len() < data.len() {
            Ok(CompressedData {
                data: compressed,
                is_compressed: true,
                original_size: data.len(),
            })
        } else {
            Ok(CompressedData {
                data: data.to_vec(),
                is_compressed: false,
                original_size: data.len(),
            })
        }
    }

    /// Decompress data
    ///
    /// If `is_compressed` is false, returns the data as-is.
    pub fn decompress(&self, data: &[u8], is_compressed: bool) -> Result<Vec<u8>> {
        if !is_compressed {
            return Ok(data.to_vec());
        }

        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder
            .read_to_end(&mut decompressed)
            .map_err(|e| CacheError::Compression {
                message: format!("Failed to decompress data: {}", e),
            })?;

        Ok(decompressed)
    }

    /// Calculate compression ratio
    pub fn compression_ratio(original_size: usize, compressed_size: usize) -> f64 {
        if original_size == 0 {
            1.0
        } else {
            compressed_size as f64 / original_size as f64
        }
    }

    /// Calculate space savings percentage
    pub fn space_savings(original_size: usize, compressed_size: usize) -> f64 {
        if original_size == 0 {
            0.0
        } else {
            (1.0 - (compressed_size as f64 / original_size as f64)) * 100.0
        }
    }
}

impl Default for CacheCompressor {
    fn default() -> Self {
        Self::new()
    }
}

/// Compressed data with metadata
#[derive(Debug, Clone)]
pub struct CompressedData {
    /// The (possibly compressed) data
    pub data: Vec<u8>,
    /// Whether the data is compressed
    pub is_compressed: bool,
    /// Original size before compression
    pub original_size: usize,
}

impl CompressedData {
    /// Get the compressed size
    pub fn compressed_size(&self) -> usize {
        self.data.len()
    }

    /// Get the compression ratio (compressed / original)
    pub fn compression_ratio(&self) -> f64 {
        CacheCompressor::compression_ratio(self.original_size, self.data.len())
    }

    /// Get space savings percentage
    pub fn space_savings(&self) -> f64 {
        CacheCompressor::space_savings(self.original_size, self.data.len())
    }
}

/// Compression statistics
#[derive(Debug, Clone, Default)]
pub struct CompressionStats {
    /// Total bytes before compression
    pub total_original_bytes: u64,
    /// Total bytes after compression
    pub total_compressed_bytes: u64,
    /// Number of entries compressed
    pub entries_compressed: u64,
    /// Number of entries not compressed (too small or no gain)
    pub entries_not_compressed: u64,
}

impl CompressionStats {
    /// Overall compression ratio
    pub fn overall_ratio(&self) -> f64 {
        if self.total_original_bytes == 0 {
            1.0
        } else {
            self.total_compressed_bytes as f64 / self.total_original_bytes as f64
        }
    }

    /// Overall space savings percentage
    pub fn overall_savings(&self) -> f64 {
        if self.total_original_bytes == 0 {
            0.0
        } else {
            (1.0 - (self.total_compressed_bytes as f64 / self.total_original_bytes as f64)) * 100.0
        }
    }

    /// Update stats with a compression result
    pub fn record(&mut self, original_size: usize, compressed: &CompressedData) {
        self.total_original_bytes += original_size as u64;
        self.total_compressed_bytes += compressed.data.len() as u64;
        if compressed.is_compressed {
            self.entries_compressed += 1;
        } else {
            self.entries_not_compressed += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompresses_correctly() {
        let compressor = CacheCompressor::new();

        // Create some compressible data (repetitive)
        let original = "Hello, World! ".repeat(100);
        let original_bytes = original.as_bytes();

        let compressed = compressor.compress(original_bytes).unwrap();
        assert!(compressed.is_compressed, "Data should be compressed");
        assert!(
            compressed.data.len() < original_bytes.len(),
            "Compressed size should be smaller"
        );

        let decompressed = compressor.decompress(&compressed.data, compressed.is_compressed).unwrap();
        assert_eq!(decompressed, original_bytes);
    }

    #[test]
    fn test_small_data_not_compressed() {
        let compressor = CacheCompressor::new().with_min_size(100);

        let small_data = b"Small";
        let result = compressor.compress(small_data).unwrap();

        assert!(!result.is_compressed);
        assert_eq!(result.data, small_data);
    }

    #[test]
    fn test_no_compression_level() {
        let compressor = CacheCompressor::with_level(CompressionLevel::None);

        let data = "Hello, World! ".repeat(100);
        let result = compressor.compress(data.as_bytes()).unwrap();

        assert!(!result.is_compressed);
        assert_eq!(result.data, data.as_bytes());
    }

    #[test]
    fn test_incompressible_data() {
        let compressor = CacheCompressor::new().with_min_size(0);

        // Random-ish data that doesn't compress well
        let data: Vec<u8> = (0..200).collect();
        let result = compressor.compress(&data).unwrap();

        // Should not mark as compressed if it didn't save space
        if result.is_compressed {
            assert!(
                result.data.len() < data.len(),
                "Compressed data should be smaller if marked as compressed"
            );
        }
    }

    #[test]
    fn test_compression_stats() {
        let mut stats = CompressionStats::default();

        let compressed1 = CompressedData {
            data: vec![0; 50],
            is_compressed: true,
            original_size: 100,
        };
        stats.record(100, &compressed1);

        let compressed2 = CompressedData {
            data: vec![0; 100],
            is_compressed: false,
            original_size: 100,
        };
        stats.record(100, &compressed2);

        assert_eq!(stats.entries_compressed, 1);
        assert_eq!(stats.entries_not_compressed, 1);
        assert_eq!(stats.total_original_bytes, 200);
        assert_eq!(stats.total_compressed_bytes, 150);
        assert!((stats.overall_savings() - 25.0).abs() < 0.01);
    }

    #[test]
    fn test_compression_levels() {
        let data = "Hello, World! ".repeat(1000);

        let fast = CacheCompressor::with_level(CompressionLevel::Fast)
            .compress(data.as_bytes())
            .unwrap();
        let default = CacheCompressor::with_level(CompressionLevel::Default)
            .compress(data.as_bytes())
            .unwrap();
        let best = CacheCompressor::with_level(CompressionLevel::Best)
            .compress(data.as_bytes())
            .unwrap();

        // Best should produce smallest output (or same as default)
        assert!(best.data.len() <= default.data.len());
        // All should be smaller than original
        assert!(fast.data.len() < data.len());
        assert!(default.data.len() < data.len());
        assert!(best.data.len() < data.len());
    }

    #[test]
    fn test_space_savings_calculation() {
        assert!((CacheCompressor::space_savings(100, 50) - 50.0).abs() < 0.01);
        assert!((CacheCompressor::space_savings(100, 100) - 0.0).abs() < 0.01);
        assert!((CacheCompressor::space_savings(100, 0) - 100.0).abs() < 0.01);
        assert!((CacheCompressor::space_savings(0, 0) - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_compression_ratio_calculation() {
        assert!((CacheCompressor::compression_ratio(100, 50) - 0.5).abs() < 0.01);
        assert!((CacheCompressor::compression_ratio(100, 100) - 1.0).abs() < 0.01);
        assert!((CacheCompressor::compression_ratio(0, 0) - 1.0).abs() < 0.01);
    }
}
