use std::collections::HashMap;
use thiserror::Error;

/// Error types for metering
#[derive(Debug, Error)]
pub enum MeteringError {
    #[error("Gas limit exceeded")]
    OutOfGas,

    #[error("Proof size limit exceeded")]
    ProofSizeLimitExceeded,

    #[error("Storage deposit limit exceeded")]
    StorageDepositLimitExceeded,
}

/// Gas costs for various operations
#[derive(Debug, Clone, Copy)]
pub struct GasCosts {
    /// Base cost for any execution
    pub base: u64,

    /// Cost per byte of input data
    pub input_byte: u64,

    /// Cost per byte of output data
    pub output_byte: u64,

    /// Cost for a storage read
    pub storage_read: u64,

    /// Cost for a storage write
    pub storage_write: u64,

    /// Cost per byte of storage write
    pub storage_write_byte: u64,

    /// Cost for a storage delete
    pub storage_delete: u64,

    /// Cost for emitting an event
    pub event: u64,

    /// Cost per byte of event data
    pub event_byte: u64,

    /// Cost for a contract call
    pub call: u64,

    /// Cost for value transfer
    pub value_transfer: u64,

    /// Cost for memory allocation
    pub memory_alloc: u64,

    /// Cost per byte of memory allocation
    pub memory_alloc_byte: u64,

    /// Cost for instruction execution
    pub instruction: u64,
}

impl Default for GasCosts {
    fn default() -> Self {
        GasCosts {
            base: 1_000,
            input_byte: 1,
            output_byte: 1,
            storage_read: 100,
            storage_write: 1_000,
            storage_write_byte: 10,
            storage_delete: 500,
            event: 100,
            event_byte: 5,
            call: 5_000,
            value_transfer: 10_000,
            memory_alloc: 10,
            memory_alloc_byte: 1,
            instruction: 1,
        }
    }
}

/// Proof size costs for various operations
#[derive(Debug, Clone, Copy)]
pub struct ProofSizeCosts {
    /// Base cost for any execution
    pub base: u64,

    /// Cost per byte of input data
    pub input_byte: u64,

    /// Cost per byte of output data
    pub output_byte: u64,

    /// Cost for a storage read
    pub storage_read: u64,

    /// Cost per byte of key in storage read
    pub storage_read_key_byte: u64,

    /// Cost for a storage write
    pub storage_write: u64,

    /// Cost per byte of key in storage write
    pub storage_write_key_byte: u64,

    /// Cost per byte of value in storage write
    pub storage_write_value_byte: u64,

    /// Cost for a storage delete
    pub storage_delete: u64,

    /// Cost per byte of key in storage delete
    pub storage_delete_key_byte: u64,

    /// Cost for emitting an event
    pub event: u64,

    /// Cost per byte of event data
    pub event_byte: u64,

    /// Cost for a contract call
    pub call: u64,

    /// Cost for memory allocation
    pub memory_alloc: u64,
}

impl Default for ProofSizeCosts {
    fn default() -> Self {
        ProofSizeCosts {
            base: 100,
            input_byte: 0,
            output_byte: 0,
            storage_read: 10,
            storage_read_key_byte: 1,
            storage_write: 10,
            storage_write_key_byte: 1,
            storage_write_value_byte: 1,
            storage_delete: 10,
            storage_delete_key_byte: 1,
            event: 10,
            event_byte: 1,
            call: 500,
            memory_alloc: 0,
        }
    }
}

/// Storage deposit costs for various operations
#[derive(Debug, Clone, Copy)]
pub struct StorageDepositCosts {
    /// Cost per byte of storage
    pub byte: u128,
}

impl Default for StorageDepositCosts {
    fn default() -> Self {
        StorageDepositCosts {
            byte: 100_000, // 0.0001 token per byte
        }
    }
}

/// Metering context for tracking resources
#[derive(Clone, Debug)]
pub struct MeteringContext {
    /// Gas costs
    pub gas_costs: GasCosts,

    /// Proof size costs
    pub proof_size_costs: ProofSizeCosts,

    /// Storage deposit costs
    pub storage_deposit_costs: StorageDepositCosts,

    /// Gas limit
    pub gas_limit: u64,

    /// Current gas used
    pub gas_used: u64,

    /// Proof size limit
    pub proof_size_limit: u64,

    /// Current proof size used
    pub proof_size_used: u64,

    /// Storage deposit limit
    pub storage_deposit_limit: u128,

    /// Current storage deposit used
    pub storage_deposit_used: u128,

    /// Instruction count
    pub instruction_count: u64,

    /// Storage size (key -> size)
    pub storage_sizes: HashMap<Vec<u8>, usize>,
}

impl MeteringContext {
    /// Create a new metering context
    pub fn new(gas_limit: u64, proof_size_limit: u64, storage_deposit_limit: u128) -> Self {
        MeteringContext {
            gas_costs: GasCosts::default(),
            proof_size_costs: ProofSizeCosts::default(),
            storage_deposit_costs: StorageDepositCosts::default(),
            gas_limit,
            gas_used: 0,
            proof_size_limit,
            proof_size_used: 0,
            storage_deposit_limit,
            storage_deposit_used: 0,
            instruction_count: 0,
            storage_sizes: HashMap::new(),
        }
    }

    /// Charge gas for an operation
    pub fn charge_gas(&mut self, amount: u64) -> Result<(), MeteringError> {
        // SECURITY FIX: Use saturating arithmetic to prevent overflow
        if amount > self.gas_limit.saturating_sub(self.gas_used) {
            return Err(MeteringError::OutOfGas);
        }
        self.gas_used = self.gas_used.saturating_add(amount);
        Ok(())
    }

    /// Charge proof size for an operation
    pub fn charge_proof_size(&mut self, amount: u64) -> Result<(), MeteringError> {
        // SECURITY FIX: Use saturating arithmetic to prevent overflow
        if amount > self.proof_size_limit.saturating_sub(self.proof_size_used) {
            return Err(MeteringError::ProofSizeLimitExceeded);
        }
        self.proof_size_used = self.proof_size_used.saturating_add(amount);
        Ok(())
    }

    /// Charge storage deposit for an operation
    pub fn charge_storage_deposit(&mut self, amount: u128) -> Result<(), MeteringError> {
        // SECURITY FIX: Use saturating arithmetic to prevent overflow
        if amount
            > self
                .storage_deposit_limit
                .saturating_sub(self.storage_deposit_used)
        {
            return Err(MeteringError::StorageDepositLimitExceeded);
        }
        self.storage_deposit_used = self.storage_deposit_used.saturating_add(amount);
        Ok(())
    }

    /// Charge resources for a storage read
    pub fn charge_storage_read(&mut self, key: &[u8]) -> Result<(), MeteringError> {
        // Charge gas
        self.charge_gas(self.gas_costs.storage_read)?;

        // Charge proof size
        self.charge_proof_size(
            self.proof_size_costs.storage_read
                + (key.len() as u64 * self.proof_size_costs.storage_read_key_byte),
        )?;

        Ok(())
    }

    /// Charge resources for a storage write
    pub fn charge_storage_write(&mut self, key: &[u8], value: &[u8]) -> Result<(), MeteringError> {
        // Charge gas
        self.charge_gas(
            self.gas_costs.storage_write + (value.len() as u64 * self.gas_costs.storage_write_byte),
        )?;

        // Charge proof size
        self.charge_proof_size(
            self.proof_size_costs.storage_write
                + (key.len() as u64 * self.proof_size_costs.storage_write_key_byte)
                + (value.len() as u64 * self.proof_size_costs.storage_write_value_byte),
        )?;

        // Charge storage deposit
        let old_size = self.storage_sizes.get(key).cloned().unwrap_or(0);
        let new_size = value.len();

        if new_size > old_size {
            // More storage is used, pay for the difference
            let deposit_amount = ((new_size - old_size) as u128) * self.storage_deposit_costs.byte;
            self.charge_storage_deposit(deposit_amount)?;
        }

        // Update storage size
        self.storage_sizes.insert(key.to_vec(), new_size);

        Ok(())
    }

    /// Charge resources for a storage delete
    pub fn charge_storage_delete(&mut self, key: &[u8]) -> Result<(), MeteringError> {
        // Charge gas
        self.charge_gas(self.gas_costs.storage_delete)?;

        // Charge proof size
        self.charge_proof_size(
            self.proof_size_costs.storage_delete
                + (key.len() as u64 * self.proof_size_costs.storage_delete_key_byte),
        )?;

        // Refund storage deposit
        if let Some(old_size) = self.storage_sizes.remove(key) {
            let refund = (old_size as u128) * self.storage_deposit_costs.byte;
            self.storage_deposit_used = self.storage_deposit_used.saturating_sub(refund);
        }

        Ok(())
    }

    /// Charge resources for an event
    pub fn charge_event(&mut self, topics: &[Vec<u8>], data: &[u8]) -> Result<(), MeteringError> {
        let mut total_size = data.len();
        for topic in topics {
            total_size += topic.len();
        }

        // Charge gas
        self.charge_gas(self.gas_costs.event + (total_size as u64 * self.gas_costs.event_byte))?;

        // Charge proof size
        self.charge_proof_size(
            self.proof_size_costs.event + (total_size as u64 * self.proof_size_costs.event_byte),
        )?;

        Ok(())
    }

    /// Charge resources for a contract call
    pub fn charge_call(&mut self, input: &[u8], value: u128) -> Result<(), MeteringError> {
        // Charge gas
        let call_gas = self.gas_costs.call + (input.len() as u64 * self.gas_costs.input_byte);

        // Add value transfer cost if applicable
        let call_gas = if value > 0 {
            call_gas + self.gas_costs.value_transfer
        } else {
            call_gas
        };

        self.charge_gas(call_gas)?;

        // Charge proof size
        self.charge_proof_size(self.proof_size_costs.call)?;

        Ok(())
    }

    /// Charge resources for a memory allocation
    pub fn charge_memory_alloc(&mut self, size: usize) -> Result<(), MeteringError> {
        // Charge gas
        self.charge_gas(
            self.gas_costs.memory_alloc + (size as u64 * self.gas_costs.memory_alloc_byte),
        )?;

        // Memory allocation doesn't affect proof size or storage deposit

        Ok(())
    }

    /// Charge resources for instruction execution
    pub fn charge_instruction(&mut self, count: u64) -> Result<(), MeteringError> {
        // Charge gas
        self.charge_gas(count * self.gas_costs.instruction)?;

        // Update instruction count
        self.instruction_count += count;

        Ok(())
    }
}
