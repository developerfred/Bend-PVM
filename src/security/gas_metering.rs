/// Gas Metering module
///
/// Provides comprehensive gas limit enforcement, tracking, and optimization
/// to prevent DoS attacks and ensure fair resource allocation.
use crate::compiler::parser::ast::*;
use crate::security::SecurityError;
use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

/// Gas cost estimation for different operations
#[derive(Debug, Clone, Copy)]
pub enum GasCost {
    Return = 0,
    Revert = 1,
    Base = 2,
    VeryLow = 3,
    Copy = 4,
    Low = 5,
    Sha3Word = 6,
    Mid = 8,
    High = 10,
    Call = 100,
    CallCode = 120,
    DelegateCall = 140,
    StaticCall = 160,
    Ext = 20,
    Sha3 = 30,
    Balance = 40,
    CodeLoad = 60,
    ExtCodeSize = 80,
    Log0 = 375,
    Log1 = 750,
    Log2 = 1125,
    Log3 = 1500,
    Log4 = 1875,
    Create = 20000,
    Create2 = 32000,
    SelfDestruct = 5000,
    Invalid = 7, // Using 7 which is not assigned to any other variant
}

/// Gas tracking entry
#[derive(Debug, Clone)]
pub struct GasEntry {
    pub function: String,
    pub gas_used: u64,
    pub timestamp: u64,
}

/// Gas limit configuration
#[derive(Debug, Clone)]
pub struct GasConfig {
    pub block_gas_limit: u64,
    pub transaction_gas_limit: u64,
    pub function_gas_limit: u64,
    pub enable_dynamic_limit: bool,
    pub gas_price_oracle: Option<GasPriceOracle>,
}

/// Gas price oracle for dynamic pricing
#[derive(Debug, Clone)]
pub struct GasPriceOracle {
    pub base_price: u64,
    pub volatility_factor: f64,
    pub min_price: u64,
    pub max_price: u64,
}

/// Gas metering system
pub struct GasMeter {
    total_gas_used: u64,
    remaining_gas: u64,
    gas_limit: u64,
    gas_history: VecDeque<GasEntry>,
    function_gas_used: HashMap<String, u64>,
    config: GasConfig,
    gas_refunds: u64,
    is_tracking: bool,
}

impl Default for GasMeter {
    fn default() -> Self {
        Self::new(10_000_000) // Default 10M gas limit
    }
}

impl GasMeter {
    /// Create a new gas meter
    pub fn new(gas_limit: u64) -> Self {
        Self {
            total_gas_used: 0,
            remaining_gas: gas_limit,
            gas_limit,
            gas_history: VecDeque::new(),
            function_gas_used: HashMap::new(),
            config: GasConfig {
                block_gas_limit: gas_limit,
                transaction_gas_limit: gas_limit,
                function_gas_limit: gas_limit / 100, // 1% of total per function
                enable_dynamic_limit: false,
                gas_price_oracle: None,
            },
            gas_refunds: 0,
            is_tracking: false,
        }
    }

    /// Create gas meter with configuration
    pub fn new_with_config(config: GasConfig) -> Self {
        let gas_limit = config.transaction_gas_limit;
        Self {
            total_gas_used: 0,
            remaining_gas: gas_limit,
            gas_limit,
            gas_history: VecDeque::new(),
            function_gas_used: HashMap::new(),
            config,
            gas_refunds: 0,
            is_tracking: false,
        }
    }

    /// Start gas tracking
    pub fn start_tracking(&mut self) {
        self.is_tracking = true;
        self.total_gas_used = 0;
        self.remaining_gas = self.gas_limit;
        self.function_gas_used.clear();
        self.gas_history.clear();
    }

    /// Stop gas tracking
    pub fn stop_tracking(&mut self) {
        self.is_tracking = false;
    }

    /// Consume gas for an operation
    pub fn consume_gas(&mut self, amount: u64) -> Result<(), SecurityError> {
        if !self.is_tracking {
            return Ok(());
        }

        if amount > self.remaining_gas {
            return Err(SecurityError::GasLimitExceeded(self.remaining_gas));
        }

        self.total_gas_used += amount;
        self.remaining_gas -= amount;

        Ok(())
    }

    /// Consume gas for a specific operation type
    pub fn consume_operation_gas(&mut self, operation: GasCost) -> Result<(), SecurityError> {
        self.consume_gas(operation as u64)
    }

    /// Consume gas for a function call
    pub fn consume_function_gas(
        &mut self,
        function: &str,
        gas_amount: u64,
    ) -> Result<(), SecurityError> {
        // Check function-specific gas limit
        let current_function_gas = self.function_gas_used.get(function).copied().unwrap_or(0);
        if current_function_gas + gas_amount > self.config.function_gas_limit {
            return Err(SecurityError::GasLimitExceeded(
                self.config.function_gas_limit - current_function_gas,
            ));
        }

        self.consume_gas(gas_amount)?;

        // Track function-specific gas usage
        *self
            .function_gas_used
            .entry(function.to_string())
            .or_insert(0) += gas_amount;

        Ok(())
    }

    /// Add gas refund
    pub fn refund_gas(&mut self, amount: u64) {
        self.gas_refunds += amount;
        self.remaining_gas += amount;
    }

    /// Estimate gas for an operation
    pub fn estimate_gas(&self, operation: GasCost) -> u64 {
        let base_cost = operation as u64;

        // Add complexity factor based on current gas usage
        let complexity_factor = 1.0 + (self.total_gas_used as f64 / self.gas_limit as f64);
        (base_cost as f64 * complexity_factor).round() as u64
    }

    /// Check gas limit before operation
    pub fn check_gas_limit(&self) -> Result<(), SecurityError> {
        if self.remaining_gas == 0 {
            return Err(SecurityError::GasLimitExceeded(0));
        }
        Ok(())
    }

    /// Get remaining gas
    pub fn remaining_gas(&self) -> u64 {
        self.remaining_gas
    }

    /// Get total gas used
    pub fn total_used(&self) -> u64 {
        self.total_gas_used
    }

    /// Get total gas consumed (alias for total_used)
    pub fn total_consumed(&self) -> u64 {
        self.total_gas_used
    }

    /// Get gas usage percentage
    pub fn gas_usage_percentage(&self) -> f64 {
        if self.gas_limit == 0 {
            0.0
        } else {
            (self.total_gas_used as f64 / self.gas_limit as f64) * 100.0
        }
    }

    /// Get gas usage for a specific function
    pub fn get_function_gas(&self, function: &str) -> u64 {
        self.function_gas_used.get(function).copied().unwrap_or(0)
    }

    /// Get gas usage statistics
    pub fn get_gas_stats(&self) -> GasStats {
        GasStats {
            total_used: self.total_gas_used,
            remaining: self.remaining_gas,
            refunds: self.gas_refunds,
            usage_percentage: self.gas_usage_percentage(),
            functions_tracked: self.function_gas_used.len(),
            average_gas_per_function: if !self.function_gas_used.is_empty() {
                self.total_gas_used / self.function_gas_used.len() as u64
            } else {
                0
            },
        }
    }

    /// Record gas usage
    pub fn record_gas_usage(&mut self, function: &str, gas_used: u64) {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.gas_history.push_back(GasEntry {
            function: function.to_string(),
            gas_used,
            timestamp: current_time,
        });

        // Keep only last 100 entries
        if self.gas_history.len() > 100 {
            self.gas_history.pop_front();
        }
    }

    /// Get gas usage history
    pub fn get_gas_history(&self) -> &VecDeque<GasEntry> {
        &self.gas_history
    }

    /// Detect gas usage anomalies
    pub fn detect_anomalies(&self) -> Vec<GasAnomaly> {
        let mut anomalies = Vec::new();

        // Check for sudden gas spikes
        if self.gas_history.len() >= 2 {
            let recent_entries: Vec<_> = self.gas_history.iter().rev().take(5).collect();
            if recent_entries.len() >= 2 {
                let avg_recent = recent_entries.iter().map(|e| e.gas_used).sum::<u64>()
                    / recent_entries.len() as u64;
                let current = recent_entries[0].gas_used;

                if current > avg_recent * 3 && avg_recent > 0 {
                    anomalies.push(GasAnomaly {
                        anomaly_type: "Gas Spike".to_string(),
                        description: format!(
                            "Function {} uses {}x average gas",
                            recent_entries[0].function,
                            current / avg_recent
                        ),
                        severity: "HIGH".to_string(),
                    });
                }
            }
        }

        // Check for high gas usage
        if self.gas_usage_percentage() > 80.0 {
            anomalies.push(GasAnomaly {
                anomaly_type: "High Gas Usage".to_string(),
                description: format!("Gas usage at {:.1}% of limit", self.gas_usage_percentage()),
                severity: "MEDIUM".to_string(),
            });
        }

        // Check for function-specific anomalies
        for (function, gas_used) in &self.function_gas_used {
            if *gas_used > self.config.function_gas_limit / 2 {
                anomalies.push(GasAnomaly {
                    anomaly_type: "High Function Gas".to_string(),
                    description: format!(
                        "Function {} uses {:.1}% of function gas limit",
                        function,
                        (*gas_used as f64 / self.config.function_gas_limit as f64) * 100.0
                    ),
                    severity: "MEDIUM".to_string(),
                });
            }
        }

        anomalies
    }

    /// Reset gas meter
    pub fn reset(&mut self) {
        self.total_gas_used = 0;
        self.remaining_gas = self.gas_limit;
        self.gas_refunds = 0;
        self.function_gas_used.clear();
        self.gas_history.clear();
    }

    /// Set new gas limit
    pub fn set_gas_limit(&mut self, new_limit: u64) {
        let used_ratio = if self.gas_limit > 0 {
            self.total_gas_used as f64 / self.gas_limit as f64
        } else {
            0.0
        };

        self.gas_limit = new_limit;
        self.remaining_gas = new_limit - (used_ratio * new_limit as f64) as u64;
        self.config.transaction_gas_limit = new_limit;
        self.config.block_gas_limit = new_limit;
    }

    /// Calculate dynamic gas price
    pub fn calculate_dynamic_gas_price(&self) -> u64 {
        if let Some(oracle) = &self.config.gas_price_oracle {
            let base_price = oracle.base_price;
            let usage_factor = self.gas_usage_percentage() / 100.0;
            let volatility_adjustment = oracle.volatility_factor * (usage_factor - 0.5);

            let mut price = base_price as f64 * (1.0 + volatility_adjustment);
            price = price.clamp(oracle.min_price as f64, oracle.max_price as f64);

            price.round() as u64
        } else {
            20_000_000_000 // Default 20 Gwei
        }
    }
}

/// Gas statistics
#[derive(Debug, Clone)]
pub struct GasStats {
    pub total_used: u64,
    pub remaining: u64,
    pub refunds: u64,
    pub usage_percentage: f64,
    pub functions_tracked: usize,
    pub average_gas_per_function: u64,
}

/// Gas anomaly detection
#[derive(Debug, Clone)]
pub struct GasAnomaly {
    pub anomaly_type: String,
    pub description: String,
    pub severity: String,
}

/// Register gas metering functions in AST
pub fn register_gas_functions() -> Vec<Definition> {
    let mut definitions = Vec::new();
    let dummy_loc = Location {
        line: 0,
        column: 0,
        start: 0,
        end: 0,
    };

    let int_type = Type::Named {
        name: "Int".to_string(),
        params: Vec::new(),
        location: dummy_loc.clone(),
    };

    let string_type = Type::Named {
        name: "String".to_string(),
        params: Vec::new(),
        location: dummy_loc.clone(),
    };

    // Get remaining gas
    definitions.push(Definition::FunctionDef {
        name: "GasMeter/remaining".to_string(),
        params: vec![],
        return_type: Some(int_type.clone()),
        body: Block {
            statements: Vec::new(),
            location: dummy_loc.clone(),
        },
        checked: Some(true),
        location: dummy_loc.clone(),
    });

    // Get gas usage percentage
    definitions.push(Definition::FunctionDef {
        name: "GasMeter/usage".to_string(),
        params: vec![],
        return_type: Some(int_type.clone()),
        body: Block {
            statements: Vec::new(),
            location: dummy_loc.clone(),
        },
        checked: Some(true),
        location: dummy_loc.clone(),
    });

    // Estimate gas for operation
    definitions.push(Definition::FunctionDef {
        name: "GasMeter/estimate".to_string(),
        params: vec![Parameter {
            name: "operation".to_string(),
            ty: string_type.clone(),
            location: dummy_loc.clone(),
        }],
        return_type: Some(int_type.clone()),
        body: Block {
            statements: Vec::new(),
            location: dummy_loc.clone(),
        },
        checked: Some(true),
        location: dummy_loc.clone(),
    });

    definitions
}
