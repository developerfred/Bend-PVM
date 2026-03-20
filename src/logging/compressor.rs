//! Log Compression
//! Issue #24 - Event System and Logging Framework

use super::logger::LogEntry;

/// Log compressor for storage optimization
pub struct LogCompressor;

impl LogCompressor {
    /// Compress log entries into a more compact representation
    /// Returns a byte vector containing the compressed data
    pub fn compress(entries: &[LogEntry]) -> Vec<u8> {
        // Simple compression - join JSON strings with newlines
        // In production, this would use actual compression (e.g., zstd, lz4)
        let json_strings: Vec<String> = entries.iter().filter_map(|e| e.to_json().ok()).collect();

        json_strings.join("\n").into_bytes()
    }

    /// Decompress log entries from compressed data
    pub fn decompress(data: &[u8]) -> Vec<LogEntry> {
        let json_string = String::from_utf8_lossy(data);
        json_string
            .lines()
            .filter_map(|line| serde_json::from_str::<LogEntry>(line).ok())
            .collect()
    }

    /// Calculate compression ratio
    pub fn compression_ratio(original_size: usize, compressed_size: usize) -> f64 {
        if original_size == 0 {
            return 1.0;
        }
        compressed_size as f64 / original_size as f64
    }
}

#[cfg(test)]
mod tests {
    use super::super::logger::LogLevel;
    use super::*;

    #[test]
    fn test_compress_decompress() {
        let entries = vec![
            LogEntry::new(LogLevel::Info, "module1", "Message 1"),
            LogEntry::new(LogLevel::Debug, "module2", "Message 2"),
            LogEntry::new(LogLevel::Warning, "module3", "Message 3"),
        ];

        let compressed = LogCompressor::compress(&entries);
        let decompressed = LogCompressor::decompress(&compressed);

        assert_eq!(decompressed.len(), entries.len());
        assert_eq!(decompressed[0].message, "Message 1");
        assert_eq!(decompressed[1].message, "Message 2");
        assert_eq!(decompressed[2].message, "Message 3");
    }

    #[test]
    fn test_compression_ratio() {
        let entries = vec![LogEntry::new(
            LogLevel::Info,
            "test",
            "A".repeat(100).as_str(),
        )];

        let original_size = entries
            .iter()
            .filter_map(|e| e.to_json().ok())
            .map(|s| s.len())
            .sum::<usize>();

        let compressed = LogCompressor::compress(&entries);
        let ratio = LogCompressor::compression_ratio(original_size, compressed.len());

        // Ratio should be close to 1.0 for this simple compression
        assert!(ratio > 0.0 && ratio <= 2.0);
    }
}
