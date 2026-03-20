/// String manipulation utilities for Bend-PVM
pub struct StringUtils;

impl StringUtils {
    /// Get the length of a string
    pub fn len(s: &str) -> usize {
        s.len()
    }

    /// Check if string is empty
    pub fn is_empty(s: &str) -> bool {
        s.is_empty()
    }

    /// Convert to uppercase
    pub fn to_uppercase(s: &str) -> String {
        s.to_uppercase()
    }

    /// Convert to lowercase
    pub fn to_lowercase(s: &str) -> String {
        s.to_lowercase()
    }

    /// Trim whitespace from both ends
    pub fn trim(s: &str) -> String {
        s.trim().to_string()
    }

    /// Trim leading whitespace
    pub fn trim_start(s: &str) -> String {
        s.trim_start().to_string()
    }

    /// Trim trailing whitespace
    pub fn trim_end(s: &str) -> String {
        s.trim_end().to_string()
    }

    /// Check if string starts with prefix
    pub fn starts_with(s: &str, prefix: &str) -> bool {
        s.starts_with(prefix)
    }

    /// Check if string ends with suffix
    pub fn ends_with(s: &str, suffix: &str) -> bool {
        s.ends_with(suffix)
    }

    /// Check if string contains substring
    pub fn contains(s: &str, substring: &str) -> bool {
        s.contains(substring)
    }

    /// Find substring, return position or -1
    pub fn find(s: &str, substring: &str) -> i128 {
        s.find(substring).map(|i| i as i128).unwrap_or(-1)
    }

    /// Get substring by range [start, end)
    pub fn substring(s: &str, start: usize, end: usize) -> Option<String> {
        if start <= end && end <= s.len() {
            Some(s.get(start..end)?.to_string())
        } else {
            None
        }
    }

    /// Split string by separator
    pub fn split(s: &str, separator: &str) -> Vec<String> {
        s.split(separator).map(|s| s.to_string()).collect()
    }

    /// Join strings with separator
    pub fn join(parts: &[String], separator: &str) -> String {
        parts.join(separator)
    }

    /// Replace substring
    pub fn replace(s: &str, from: &str, to: &str) -> String {
        s.replace(from, to)
    }

    /// Repeat string n times
    pub fn repeat(s: &str, n: usize) -> String {
        s.repeat(n)
    }

    /// Reverse string
    pub fn reverse(s: &str) -> String {
        s.chars().rev().collect()
    }

    /// Convert to bytes
    pub fn to_bytes(s: &str) -> Vec<u8> {
        s.as_bytes().to_vec()
    }

    /// Create string from bytes
    pub fn from_bytes(bytes: &[u8]) -> String {
        String::from_utf8_lossy(bytes).to_string()
    }

    /// Hex encode
    pub fn hex_encode(s: &str) -> String {
        hex::encode(s.as_bytes())
    }

    /// Hex decode
    pub fn hex_decode(hex: &str) -> Option<String> {
        hex::decode(hex)
            .ok()
            .map(|bytes| String::from_utf8_lossy(&bytes).to_string())
    }

    /// Base64 encode
    pub fn base64_encode(s: &str) -> String {
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, s.as_bytes())
    }

    /// Base64 decode
    pub fn base64_decode(encoded: &str) -> Option<String> {
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encoded)
            .ok()
            .map(|bytes| String::from_utf8_lossy(&bytes).to_string())
    }

    /// SHA256 hash of string
    pub fn sha256(s: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(s.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Keccak256 hash of string
    pub fn keccak256(s: &str) -> String {
        use tiny_keccak::{Hasher, Keccak};
        let mut keccak = Keccak::v256();
        let mut output = [0u8; 32];
        keccak.update(s.as_bytes());
        keccak.finalize(&mut output);
        hex::encode(output)
    }
}

/// Format utilities
pub struct Format;

impl Format {
    /// Format string with arguments (simple placeholder: {})
    pub fn format(template: &str, args: &[String]) -> String {
        let mut result = template.to_string();
        for arg in args {
            if let Some(pos) = result.find("{}") {
                result.replace_range(pos..=pos, arg);
            } else {
                break;
            }
        }
        result
    }

    /// Format number with decimal places
    pub fn number(n: u128, decimals: usize) -> String {
        if decimals == 0 {
            n.to_string()
        } else {
            let divisor = 10u128.pow(decimals as u32);
            let integer_part = n / divisor;
            let fractional_part = n % divisor;
            format!(
                "{}.{:0width$}",
                integer_part,
                fractional_part,
                width = decimals
            )
        }
    }

    /// Format currency with symbol
    pub fn currency(amount: u128, symbol: &str, decimals: usize) -> String {
        format!("{}{}", symbol, Format::number(amount, decimals))
    }

    /// Format percentage (basis points to percent)
    pub fn percentage(bps: u128) -> String {
        let percent = bps as f64 / 100.0;
        format!("{:.2}%", percent)
    }

    /// Format address (shorten for display)
    pub fn address(addr: &str) -> String {
        if addr.len() >= 10 {
            format!("{}...{}", &addr[..6], &addr[addr.len() - 4..])
        } else {
            addr.to_string()
        }
    }

    /// Format transaction hash (shorten for display)
    pub fn tx_hash(hash: &str) -> String {
        if hash.len() >= 14 {
            format!("{}...{}", &hash[..10], &hash[hash.len() - 4..])
        } else {
            hash.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_utils_basic() {
        assert_eq!(StringUtils::len("hello"), 5);
        assert!(StringUtils::is_empty(""));
        assert!(!StringUtils::is_empty("x"));
        assert_eq!(StringUtils::to_uppercase("hello"), "HELLO");
        assert_eq!(StringUtils::to_lowercase("HELLO"), "hello");
    }

    #[test]
    fn test_string_utils_trim() {
        assert_eq!(StringUtils::trim("  hello  "), "hello");
        assert_eq!(StringUtils::trim_start("  hello"), "hello");
        assert_eq!(StringUtils::trim_end("hello  "), "hello");
    }

    #[test]
    fn test_string_utils_contains() {
        assert!(StringUtils::starts_with("hello", "he"));
        assert!(!StringUtils::starts_with("hello", "lo"));
        assert!(StringUtils::ends_with("hello", "lo"));
        assert!(!StringUtils::ends_with("hello", "he"));
        assert!(StringUtils::contains("hello", "ell"));
        assert!(!StringUtils::contains("hello", "world"));
    }

    #[test]
    fn test_string_utils_find() {
        assert_eq!(StringUtils::find("hello", "ell"), 1);
        assert_eq!(StringUtils::find("hello", "world"), -1);
    }

    #[test]
    fn test_string_utils_substring() {
        assert_eq!(
            StringUtils::substring("hello", 0, 3),
            Some("hel".to_string())
        );
        assert_eq!(StringUtils::substring("hello", 0, 10), None);
        assert_eq!(
            StringUtils::substring("hello", 3, 5),
            Some("lo".to_string())
        );
    }

    #[test]
    fn test_string_utils_split_join() {
        assert_eq!(StringUtils::split("a,b,c", ","), vec!["a", "b", "c"]);
        assert_eq!(
            StringUtils::join(&["a".to_string(), "b".to_string(), "c".to_string()], "-"),
            "a-b-c"
        );
    }

    #[test]
    fn test_string_utils_replace_repeat_reverse() {
        assert_eq!(StringUtils::replace("hello", "l", "r"), "herro");
        assert_eq!(StringUtils::repeat("ab", 3), "ababab");
        assert_eq!(StringUtils::reverse("hello"), "olleh");
    }

    #[test]
    fn test_string_utils_bytes() {
        assert_eq!(StringUtils::to_bytes("hello"), b"hello".to_vec());
        assert_eq!(StringUtils::from_bytes(b"hello"), "hello");
    }

    #[test]
    fn test_string_utils_hex() {
        let encoded = StringUtils::hex_encode("hello");
        assert_eq!(encoded, "68656c6c6f");
        assert_eq!(
            StringUtils::hex_decode("68656c6c6f"),
            Some("hello".to_string())
        );
        assert_eq!(StringUtils::hex_decode("invalid"), None);
    }

    #[test]
    fn test_string_utils_base64() {
        let encoded = StringUtils::base64_encode("hello");
        assert_eq!(encoded, "aGVsbG8=");
        assert_eq!(
            StringUtils::base64_decode("aGVsbG8="),
            Some("hello".to_string())
        );
        assert!(StringUtils::base64_decode("!!!").is_none());
    }

    #[test]
    fn test_string_utils_hash() {
        let sha = StringUtils::sha256("hello");
        assert_eq!(sha.len(), 64);

        let keccak = StringUtils::keccak256("hello");
        assert_eq!(keccak.len(), 64);
    }

    #[test]
    fn test_format_tx_hash() {
        assert_eq!(
            Format::tx_hash("0x1234567890abcdef1234"),
            "0x12345678...1234"
        );
        assert_eq!(Format::tx_hash("short"), "short");
        assert_eq!(Format::tx_hash("1234567890abcdef"), "1234567890...cdef");
    }
}
