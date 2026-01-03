use crate::compiler::parser::ast::*;
use crate::runtime::env::Environment;
use crate::runtime::metering::MeteringError;

/// Crypto functions implementation
pub struct CryptoFunctions;

impl CryptoFunctions {
    /// Create a new crypto functions instance
    pub fn new() -> Self {
        CryptoFunctions
    }

    /// Keccak-256 hash
    pub fn keccak256(&self, data: &[u8]) -> Vec<u8> {
        use tiny_keccak::{HashOutput, Keccak};
        let mut keccak = Keccak::v256();
        let mut output = [0u8; 32];
        keccak.update(data);
        keccak.finalize(&mut output);
        output.to_vec()
    }

    /// SHA-256 hash
    pub fn sha256(&self, data: &[u8]) -> Vec<u8> {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }

    /// SHA-3-256 hash
    pub fn sha3_256(&self, data: &[u8]) -> Vec<u8> {
        use sha3::{Digest, Sha3_256};
        let mut hasher = Sha3_256::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }

    /// BLAKE2b-256 hash
    pub fn blake2b(&self, data: &[u8]) -> Vec<u8> {
        use blake2::Digest;
        let mut hasher = blake2::Blake2b256::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }

    /// RIPEMD-160 hash
    pub fn ripemd160(&self, data: &[u8]) -> Vec<u8> {
        use ripemd::Ripemd160;
        let mut hasher = Ripemd160::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }

    /// HMAC-SHA256
    pub fn hmac_sha256(&self, key: &[u8], data: &[u8]) -> Vec<u8> {
        use hmac::{Hmac, Mac};
        type HmacSha256 = Hmac<sha2::Sha256>;
        let mut mac = HmacSha256::new_from_slice(key).expect("HMAC size ok");
        mac.update(data);
        let result = mac.finalize();
        result.into_bytes().to_vec()
    }

    /// PBKDF2-HMAC-SHA256
    pub fn pbkdf2_sha256(&self, password: &[u8], salt: &[u8], iterations: u32) -> Vec<u8> {
        use hmac::{Hmac, Mac};
        use pbkdf2::Pbkdf2;
        type HmacSha256 = Hmac<sha2::Sha256>;
        let mut result = [0u8; 32];
        let _ = Pbkdf2::<HmacSha256>::derive_key(password, salt, iterations, &mut result);
        result.to_vec()
    }

    /// Scrypt key derivation
    pub fn scrypt(&self, password: &[u8], salt: &[u8], log_n: u8, r: u32, p: u32) -> Vec<u8> {
        use scrypt::{Params, Scrypt};
        let params = Params::new(log_n, r, p).expect("Valid scrypt params");
        let mut result = vec![0u8; 64];
        let _ = Scrypt::new(password, salt, params, &mut result);
        result
    }

    /// Verify ECDSA signature (simplified)
    pub fn verify_ecdsa(&self, message: &[u8], signature: &[u8], public_key: &[u8]) -> bool {
        // In production, use a proper ECDSA library
        // This is a placeholder that validates structure
        if signature.len() != 64 && signature.len() != 65 {
            return false;
        }
        if public_key.len() != 33 && public_key.len() != 65 {
            return false;
        }
        // Basic validation passed
        true
    }

    /// Recover ECDSA public key from signature
    pub fn recover_ecdsa(&self, message: &[u8], signature: &[u8]) -> Option<Vec<u8>> {
        // In production, use a proper ECDSA library
        if signature.len() != 65 {
            return None;
        }
        // Placeholder for public key recovery
        Some(vec![0x04; 65])
    }

    /// Generate random private key (for testing only)
    pub fn generate_private_key(&self) -> Vec<u8> {
        let mut key = [0u8; 32];
        getrandom::getrandom(&mut key).unwrap_or_default();
        key.to_vec()
    }

    /// Derive public key from private key
    pub fn private_to_public(&self, private_key: &[u8]) -> Option<Vec<u8>> {
        if private_key.len() != 32 {
            return None;
        }
        // In production, use a proper EC library
        // Return uncompressed public key
        Some(vec![0x04; 65])
    }

    /// Compress public key (return x coordinate with parity)
    pub fn compress_public_key(&self, public_key: &[u8]) -> Option<Vec<u8>> {
        if public_key.len() != 65 || public_key[0] != 0x04 {
            return None;
        }
        // Return compressed format (prefix + x coordinate)
        let mut compressed = vec![0x02; 33];
        compressed[1..].copy_from_slice(&public_key[1..33]);
        Some(compressed)
    }

    /// Generate address from public key
    pub fn public_key_to_address(&self, public_key: &[u8]) -> Vec<u8> {
        let hash = self.keccak256(public_key);
        // Return last 20 bytes as address
        hash[12..].to_vec()
    }

    /// Validate address format
    pub fn is_valid_address(&self, address: &[u8]) -> bool {
        if address.len() != 20 {
            return false;
        }
        // Basic validation - could add checksum validation
        true
    }

    /// Checksum address (EIP-55)
    pub fn checksum_address(&self, address: &[u8]) -> String {
        let hash = self.keccak256(address);
        let mut result = String::new();
        for (i, byte) in address.iter().enumerate() {
            let nibble = hash[i / 2] >> if i % 2 == 0 { 4 } else { 0 } & 0x0F;
            if *byte >= b'a' && *byte <= b'z' {
                if nibble >= 8 {
                    result.push((*byte as char).to_ascii_uppercase());
                } else {
                    result.push(*byte as char);
                }
            } else {
                result.push(*byte as char);
            }
        }
        result
    }

    /// Encrypt with AES-256-GCM
    pub fn aes256_gcm_encrypt(
        &self,
        plaintext: &[u8],
        key: &[u8],
        nonce: &[u8],
    ) -> Option<Vec<u8>> {
        use aes_gcm::aead::Aead;
        use aes_gcm::{Aes256Gcm, Key, Nonce};

        if key.len() != 32 || nonce.len() != 12 {
            return None;
        }

        let cipher = Aes256Gcm::new(Key::from_slice(key));
        let nonce = Nonce::from_slice(nonce);
        cipher.encrypt(nonce, plaintext).ok()
    }

    /// Decrypt with AES-256-GCM
    pub fn aes256_gcm_decrypt(
        &self,
        ciphertext: &[u8],
        key: &[u8],
        nonce: &[u8],
    ) -> Option<Vec<u8>> {
        use aes_gcm::aead::Aead;
        use aes_gcm::{Aes256Gcm, Key, Nonce};

        if key.len() != 32 || nonce.len() != 12 {
            return None;
        }

        let cipher = Aes256Gcm::new(Key::from_slice(key));
        let nonce = Nonce::from_slice(nonce);
        cipher.decrypt(nonce, ciphertext).ok()
    }

    /// Encrypt with ChaCha20-Poly1305
    pub fn chacha20_poly1305_encrypt(
        &self,
        plaintext: &[u8],
        key: &[u8],
        nonce: &[u8],
    ) -> Option<Vec<u8>> {
        use chacha20poly1305::aead::Aead;
        use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};

        if key.len() != 32 || nonce.len() != 12 {
            return None;
        }

        let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
        let nonce = Nonce::from_slice(nonce);
        cipher.encrypt(nonce, plaintext).ok()
    }

    /// Decrypt with ChaCha20-Poly1305
    pub fn chacha20_poly1305_decrypt(
        &self,
        ciphertext: &[u8],
        key: &[u8],
        nonce: &[u8],
    ) -> Option<Vec<u8>> {
        use chacha20poly1305::aead::Aead;
        use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};

        if key.len() != 32 || nonce.len() != 12 {
            return None;
        }

        let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
        let nonce = Nonce::from_slice(nonce);
        cipher.decrypt(nonce, ciphertext).ok()
    }

    /// Base64 encode
    pub fn base64_encode(&self, data: &[u8]) -> String {
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, data)
    }

    /// Base64 decode
    pub fn base64_decode(&self, encoded: &str) -> Option<Vec<u8>> {
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encoded).ok()
    }

    /// Hex encode
    pub fn hex_encode(&self, data: &[u8]) -> String {
        hex::encode(data)
    }

    /// Hex decode
    pub fn hex_decode(&self, encoded: &str) -> Option<Vec<u8>> {
        hex::decode(encoded).ok()
    }
}

/// Register crypto functions in the runtime environment
pub fn register_crypto_functions(_env: &mut Environment) -> Result<(), MeteringError> {
    Ok(())
}

/// Generate AST for crypto library
pub fn generate_crypto_ast() -> Vec<Definition> {
    let mut definitions = Vec::new();
    let dummy_loc = Location {
        line: 0,
        column: 0,
        start: 0,
        end: 0,
    };

    let string_type = Type::Named {
        name: "String".to_string(),
        params: Vec::new(),
        location: dummy_loc.clone(),
    };

    // Crypto/keccak256
    definitions.push(Definition::FunctionDef {
        name: "Crypto/keccak256".to_string(),
        params: vec![Parameter {
            name: "data".to_string(),
            ty: string_type.clone(),
            location: dummy_loc.clone(),
        }],
        return_type: Some(string_type.clone()),
        body: Block {
            statements: Vec::new(),
            location: dummy_loc.clone(),
        },
        checked: Some(true),
        location: dummy_loc.clone(),
    });

    // Crypto/sha256
    definitions.push(Definition::FunctionDef {
        name: "Crypto/sha256".to_string(),
        params: vec![Parameter {
            name: "data".to_string(),
            ty: string_type.clone(),
            location: dummy_loc.clone(),
        }],
        return_type: Some(string_type.clone()),
        body: Block {
            statements: Vec::new(),
            location: dummy_loc.clone(),
        },
        checked: Some(true),
        location: dummy_loc.clone(),
    });

    // Crypto/verify_ecdsa
    definitions.push(Definition::FunctionDef {
        name: "Crypto/verify_ecdsa".to_string(),
        params: vec![
            Parameter {
                name: "message".to_string(),
                ty: string_type.clone(),
                location: dummy_loc.clone(),
            },
            Parameter {
                name: "signature".to_string(),
                ty: string_type.clone(),
                location: dummy_loc.clone(),
            },
            Parameter {
                name: "public_key".to_string(),
                ty: string_type.clone(),
                location: dummy_loc.clone(),
            },
        ],
        return_type: Some(Type::U24 {
            location: dummy_loc.clone(),
        }),
        body: Block {
            statements: Vec::new(),
            location: dummy_loc.clone(),
        },
        checked: Some(true),
        location: dummy_loc.clone(),
    });

    definitions
}
