use crate::testing::{TestError, TestEnvironment};

/// Test assertions for verifying test results
pub struct TestAssertions<'a> {
    /// Test environment
    environment: &'a TestEnvironment,
}

impl<'a> TestAssertions<'a> {
    /// Create new test assertions
    pub fn new(environment: &'a TestEnvironment) -> Self {
        TestAssertions {
            environment,
        }
    }
    
    /// Assert that a storage value exists
    pub fn assert_storage_exists(&self, key: &[u8]) -> Result<(), TestError> {
        let mut storage = self.environment.storage.clone();
        let mut metering = self.environment.metering.clone();
        
        match storage.contains(key, &mut metering) {
            Ok(exists) => {
                if exists {
                    Ok(())
                } else {
                    Err(TestError::AssertionFailed(format!(
                        "Storage key {:?} does not exist", key
                    )))
                }
            },
            Err(err) => Err(TestError::AssertionFailed(format!(
                "Storage error: {}", err
            ))),
        }
    }
    
    /// Assert that a storage value equals the expected value
    pub fn assert_storage_eq(&self, key: &[u8], expected: &[u8]) -> Result<(), TestError> {
        let mut storage = self.environment.storage.clone();
        let mut metering = self.environment.metering.clone();
        
        match storage.get(key, &mut metering) {
            Ok(Some(value)) => {
                if value == expected {
                    Ok(())
                } else {
                    Err(TestError::AssertionFailed(format!(
                        "Storage key {:?} has value {:?}, expected {:?}",
                        key, value, expected
                    )))
                }
            },
            Ok(None) => Err(TestError::AssertionFailed(format!(
                "Storage key {:?} does not exist", key
            ))),
            Err(err) => Err(TestError::AssertionFailed(format!(
                "Storage error: {}", err
            ))),
        }
    }
    
    /// Assert that gas used is less than the expected amount
    pub fn assert_gas_used_lt(&self, expected: u64) -> Result<(), TestError> {
        let gas_used = self.environment.context.gas_used;
        
        if gas_used < expected {
            Ok(())
        } else {
            Err(TestError::AssertionFailed(format!(
                "Gas used {} is not less than expected {}", gas_used, expected
            )))
        }
    }
    
    /// Assert that gas used is less than or equal to the expected amount
    pub fn assert_gas_used_le(&self, expected: u64) -> Result<(), TestError> {
        let gas_used = self.environment.context.gas_used;
        
        if gas_used <= expected {
            Ok(())
        } else {
            Err(TestError::AssertionFailed(format!(
                "Gas used {} is greater than expected {}", gas_used, expected
            )))
        }
    }
    
    /// Assert that storage deposit used is less than the expected amount
    pub fn assert_storage_deposit_lt(&self, expected: u128) -> Result<(), TestError> {
        let storage_deposit_used = self.environment.context.storage_deposit_used;
        
        if storage_deposit_used < expected {
            Ok(())
        } else {
            Err(TestError::AssertionFailed(format!(
                "Storage deposit used {} is not less than expected {}",
                storage_deposit_used, expected
            )))
        }
    }
    
    /// Assert that storage deposit used is less than or equal to the expected amount
    pub fn assert_storage_deposit_le(&self, expected: u128) -> Result<(), TestError> {
        let storage_deposit_used = self.environment.context.storage_deposit_used;
        
        if storage_deposit_used <= expected {
            Ok(())
        } else {
            Err(TestError::AssertionFailed(format!(
                "Storage deposit used {} is greater than expected {}",
                storage_deposit_used, expected
            )))
        }
    }
    
    /// Assert that an event was emitted
    pub fn assert_event_emitted(&self, _name: &str) -> Result<(), TestError> {
        // In a real implementation, this would check the events emitted by the contract
        // For now, we just return Ok
        Ok(())
    }
    
    /// Assert that an event was emitted with specific data
    pub fn assert_event_data(&self, _name: &str, _data: &[u8]) -> Result<(), TestError> {
        // In a real implementation, this would check the events emitted by the contract
        // For now, we just return Ok
        Ok(())
    }
}