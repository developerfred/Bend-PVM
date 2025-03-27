use std::collections::HashMap;
use thiserror::Error;

/// Error types for runtime environment
#[derive(Debug, Error)]
pub enum EnvError {
    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Memory error: {0}")]
    Memory(String),

    #[error("Contract execution error: {0}")]
    Execution(String),

    #[error("Function call error: {0}")]
    Call(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Gas limit exceeded")]
    OutOfGas,
}

/// Context for contract execution
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Address of the contract
    pub address: [u8; 32],

    /// Address of the caller
    pub caller: [u8; 32],

    /// Value sent with the call (in smallest units)
    pub value: u128,

    /// Input data (calldata)
    pub input: Vec<u8>,

    /// Block number
    pub block_number: u64,

    /// Block timestamp
    pub block_timestamp: u64,

    /// Gas limit for the execution
    pub gas_limit: u64,

    /// Current gas used
    pub gas_used: u64,

    /// Maximum proof size allowed
    pub proof_size_limit: u64,

    /// Current proof size used
    pub proof_size_used: u64,

    /// Storage deposit limit
    pub storage_deposit_limit: u128,

    /// Current storage deposit used
    pub storage_deposit_used: u128,
}

impl ExecutionContext {
    /// Create a new execution context
    pub fn new(
        address: [u8; 32],
        caller: [u8; 32],
        value: u128,
        input: Vec<u8>,
        block_number: u64,
        block_timestamp: u64,
        gas_limit: u64,
        proof_size_limit: u64,
        storage_deposit_limit: u128,
    ) -> Self {
        ExecutionContext {
            address,
            caller,
            value,
            input,
            block_number,
            block_timestamp,
            gas_limit,
            gas_used: 0,
            proof_size_limit,
            proof_size_used: 0,
            storage_deposit_limit,
            storage_deposit_used: 0,
        }
    }

    /// Check if there's enough gas for an operation
    pub fn check_gas(&self, gas: u64) -> Result<(), EnvError> {
        if self.gas_used + gas > self.gas_limit {
            Err(EnvError::OutOfGas)
        } else {
            Ok(())
        }
    }

    /// Use gas for an operation
    pub fn use_gas(&mut self, gas: u64) -> Result<(), EnvError> {
        self.check_gas(gas)?;
        self.gas_used += gas;
        Ok(())
    }

    /// Check if there's enough proof size for an operation
    pub fn check_proof_size(&self, size: u64) -> Result<(), EnvError> {
        if self.proof_size_used + size > self.proof_size_limit {
            Err(EnvError::Execution("Proof size limit exceeded".to_string()))
        } else {
            Ok(())
        }
    }

    /// Use proof size for an operation
    pub fn use_proof_size(&mut self, size: u64) -> Result<(), EnvError> {
        self.check_proof_size(size)?;
        self.proof_size_used += size;
        Ok(())
    }

    /// Check if there's enough storage deposit for an operation
    pub fn check_storage_deposit(&self, amount: u128) -> Result<(), EnvError> {
        if self.storage_deposit_used + amount > self.storage_deposit_limit {
            Err(EnvError::Execution("Storage deposit limit exceeded".to_string()))
        } else {
            Ok(())
        }
    }

    /// Use storage deposit for an operation
    pub fn use_storage_deposit(&mut self, amount: u128) -> Result<(), EnvError> {
        self.check_storage_deposit(amount)?;
        self.storage_deposit_used += amount;
        Ok(())
    }
}

/// Result of contract execution
#[derive(Debug, Clone)]
pub enum ExecutionResult {
    /// Successful execution with return data
    Success {
        /// Return data
        data: Vec<u8>,
        /// Gas used
        gas_used: u64,
        /// Proof size used
        proof_size_used: u64,
        /// Storage deposit used
        storage_deposit_used: u128,
    },

    /// Execution failure with reason
    Failure {
        /// Error reason
        reason: String,
        /// Gas used
        gas_used: u64,
        /// Proof size used
        proof_size_used: u64,
        /// Storage deposit used
        storage_deposit_used: u128,
    },

    /// Execution reverted with data
    Revert {
        /// Revert data
        data: Vec<u8>,
        /// Gas used
        gas_used: u64,
        /// Proof size used
        proof_size_used: u64,
        /// Storage deposit used
        storage_deposit_used: u128,
    },
}

/// Event emitted by a contract
#[derive(Debug, Clone)]
pub struct Event {
    /// Event topics (indexed parameters)
    pub topics: Vec<Vec<u8>>,
    /// Event data (non-indexed parameters)
    pub data: Vec<u8>,
}

/// Runtime environment for contract execution
pub struct Environment {
    /// Contract storage
    pub storage: HashMap<Vec<u8>, Vec<u8>>,
    /// Emitted events
    pub events: Vec<Event>,
    /// Execution context
    pub context: ExecutionContext,
}

impl Environment {
    /// Create a new environment
    pub fn new(context: ExecutionContext) -> Self {
        Environment {
            storage: HashMap::new(),
            events: Vec::new(),
            context,
        }
    }

    /// Read from storage
    pub fn storage_get(&mut self, key: &[u8]) -> Result<Option<Vec<u8>>, EnvError> {
        // Use gas for the operation
        self.context.use_gas(200)?;
        
        // Use proof size for the operation
        // Accessing storage requires proving the key, so we add the key size to proof size
        self.context.use_proof_size(key.len() as u64)?;
        
        // Return the value
        Ok(self.storage.get(key).cloned())
    }

    /// Write to storage
    pub fn storage_set(&mut self, key: &[u8], value: &[u8]) -> Result<(), EnvError> {
        // Use gas for the operation
        self.context.use_gas(200 + value.len() as u64)?;
        
        // Use proof size for the operation
        self.context.use_proof_size((key.len() + value.len()) as u64)?;
        
        // Calculate storage deposit
        // If we're replacing an existing value, we only pay for the difference
        let old_size = self.storage.get(key).map_or(0, |v| v.len());
        let new_size = value.len();
        
        if new_size > old_size {
            // More storage is used, pay for the difference
            let deposit_amount = (new_size - old_size) as u128;
            self.context.use_storage_deposit(deposit_amount)?;
        }
        
        // Store the value
        self.storage.insert(key.to_vec(), value.to_vec());
        
        Ok(())
    }

    /// Delete from storage
    pub fn storage_clear(&mut self, key: &[u8]) -> Result<(), EnvError> {
        // Use gas for the operation
        self.context.use_gas(200)?;
        
        // Use proof size for the operation
        self.context.use_proof_size(key.len() as u64)?;
        
        // Refund storage deposit (in a real implementation this would go back to the caller)
        if let Some(old_value) = self.storage.get(key) {
            // This is a refund, so we don't check limits
            self.context.storage_deposit_used = self.context.storage_deposit_used.saturating_sub(old_value.len() as u128);
        }
        
        // Remove the value
        self.storage.remove(key);
        
        Ok(())
    }

    /// Emit an event
    pub fn emit_event(&mut self, topics: Vec<Vec<u8>>, data: Vec<u8>) -> Result<(), EnvError> {
        // Check event limitations
        if topics.len() > 4 {
            return Err(EnvError::InvalidInput("Too many event topics".to_string()));
        }
        
        let mut total_size = data.len();
        for topic in &topics {
            total_size += topic.len();
        }
        
        if total_size > 416 {
            return Err(EnvError::InvalidInput("Event too large".to_string()));
        }
        
        // Use gas for the operation
        self.context.use_gas(100 + total_size as u64)?;
        
        // Use proof size for the operation
        self.context.use_proof_size(total_size as u64)?;
        
        // Emit the event
        self.events.push(Event {
            topics,
            data,
        });
        
        Ok(())
    }

    /// Call another contract
    pub fn call(
        &mut self,
        address: [u8; 32],
        value: u128,
        input: Vec<u8>,
        gas_limit: u64,
        proof_size_limit: u64,
        storage_deposit_limit: u128,
    ) -> Result<ExecutionResult, EnvError> {
        // In a real implementation, this would execute the contract at the given address
        // For this example, we'll just simulate it
        
        // Use gas for the operation
        self.context.use_gas(100 + input.len() as u64)?;
        
        // Use proof size for the operation
        self.context.use_proof_size(input.len() as u64)?;
        
        // Check if there's enough gas for the call
        if self.context.gas_used + gas_limit > self.context.gas_limit {
            return Err(EnvError::OutOfGas);
        }
        
        // Check value transfer
        if value > 0 {
            return Err(EnvError::Call("Value transfer not supported in this example".to_string()));
        }
        
        // Simulate the call - in a real implementation, this would use PolkaVM to execute the contract
        let result = ExecutionResult::Success {
            data: vec![1, 2, 3, 4], // Some dummy data
            gas_used: gas_limit / 2, // Use half the provided gas
            proof_size_used: proof_size_limit / 2, // Use half the provided proof size
            storage_deposit_used: storage_deposit_limit / 2, // Use half the provided storage deposit
        };
        
        // Update gas used
        match &result {
            ExecutionResult::Success { gas_used, proof_size_used, storage_deposit_used, .. } |
            ExecutionResult::Failure { gas_used, proof_size_used, storage_deposit_used, .. } |
            ExecutionResult::Revert { gas_used, proof_size_used, storage_deposit_used, .. } => {
                self.context.gas_used += gas_used;
                self.context.proof_size_used += proof_size_used;
                self.context.storage_deposit_used += storage_deposit_used;
            }
        }
        
        Ok(result)
    }

    /// Execute the current contract
    pub fn execute(&mut self, code: &[u8]) -> Result<ExecutionResult, EnvError> {
        // In a real implementation, this would use PolkaVM to execute the contract
        // For this example, we'll just return a dummy result
        
        // Simulate execution
        if self.context.input.starts_with(&[0xDE, 0xAD, 0xBE, 0xEF]) {
            // Simulate a revert
            Ok(ExecutionResult::Revert {
                data: vec![0xFA, 0x1L, 0xED],
                gas_used: self.context.gas_limit / 4,
                proof_size_used: self.context.proof_size_limit / 4,
                storage_deposit_used: self.context.storage_deposit_limit / 4,
            })
        } else {
            // Simulate a successful execution
            Ok(ExecutionResult::Success {
                data: vec![0, 1, 2, 3],
                gas_used: self.context.gas_limit / 2,
                proof_size_used: self.context.proof_size_limit / 2,
                storage_deposit_used: self.context.storage_deposit_limit / 2,
            })
        }
    }
}