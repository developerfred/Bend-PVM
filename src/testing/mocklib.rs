use std::collections::HashMap;

/// Mock standard library for testing
pub struct MockStdlib {
    /// Mock responses for external calls
    pub call_responses: HashMap<String, Vec<u8>>,
    
    /// Mock responses for storage gets
    pub storage_responses: HashMap<Vec<u8>, Vec<u8>>,
}

impl MockStdlib {
    /// Create a new mock standard library
    pub fn new() -> Self {
        MockStdlib {
            call_responses: HashMap::new(),
            storage_responses: HashMap::new(),
        }
    }
    
    /// Mock a response for an external call
    pub fn mock_call(&mut self, address: &str, data: &[u8], response: Vec<u8>) {
        let key = Self::call_key(address, data);
        self.call_responses.insert(key, response);
    }
    
    /// Mock a response for a storage get
    pub fn mock_storage(&mut self, key: &[u8], value: Vec<u8>) {
        self.storage_responses.insert(key.to_vec(), value);
    }
    
    /// Get a mock response for an external call
    pub fn get_call_response(&self, address: &str, data: &[u8]) -> Option<Vec<u8>> {
        let key = Self::call_key(address, data);
        self.call_responses.get(&key).cloned()
    }
    
    /// Get a mock response for a storage get
    pub fn get_storage_response(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.storage_responses.get(key).cloned()
    }
    
    /// Create a key for call mocking
    fn call_key(address: &str, data: &[u8]) -> String {
        format!("{}:{}", address, hex::encode(data))
    }
    
    /// Mock keccak256 hash
    pub fn keccak256(&self, data: &[u8]) -> Vec<u8> {
        // In a real implementation, this would compute a keccak256 hash
        // For testing, we'll just return a fixed value
        vec![0x12, 0x34, 0x56, 0x78]
    }
    
    /// Mock sha256 hash
    pub fn sha256(&self, data: &[u8]) -> Vec<u8> {
        // In a real implementation, this would compute a sha256 hash
        // For testing, we'll just return a fixed value
        vec![0x87, 0x65, 0x43, 0x21]
    }
    
    /// Mock ECDSA recovery
    pub fn ecdsa_recover(&self, hash: &[u8], signature: &[u8]) -> Option<Vec<u8>> {
        // In a real implementation, this would recover a public key from a signature
        // For testing, we'll just return a fixed value
        Some(vec![0x42, 0x42, 0x42, 0x42])
    }
    
    /// Mock random number generation
    pub fn random(&self, seed: &[u8]) -> Vec<u8> {
        // In a real implementation, this would generate a random number
        // For testing, we'll just return a fixed value
        vec![0x12, 0x34, 0x56, 0x78]
    }
}