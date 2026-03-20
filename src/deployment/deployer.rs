//! Contract Deployer
//! Issue #23 - Contract Deployment and Management Tools

use super::config::DeploymentConfig;
use super::state::{DeploymentState, DeploymentStatus};

/// Contract deployer for managing deployments
#[derive(Debug)]
pub struct ContractDeployer {
    config: DeploymentConfig,
    state: DeploymentState,
}

impl ContractDeployer {
    pub fn new(config: DeploymentConfig) -> Self {
        ContractDeployer {
            config,
            state: DeploymentState::new(),
        }
    }

    pub fn get_state(&self) -> &DeploymentState {
        &self.state
    }

    pub fn get_config(&self) -> &DeploymentConfig {
        &self.config
    }

    pub fn validate(&mut self) -> Result<(), String> {
        self.state.set_status(DeploymentStatus::Validating);

        if self.config.gas_limit == 0 {
            return Err("Gas limit must be greater than 0".to_string());
        }

        if self.config.timeout_seconds == 0 {
            return Err("Timeout must be greater than 0".to_string());
        }

        if self.config.network.rpc_url.is_empty() {
            return Err("RPC URL cannot be empty".to_string());
        }

        Ok(())
    }

    pub fn prepare(&mut self) -> Result<String, String> {
        if self.config.dry_run {
            return Ok("DRY_RUN".to_string());
        }

        self.state.set_status(DeploymentStatus::Deploying);
        Ok("READY".to_string())
    }

    pub fn deploy(&mut self, _bytecode: &[u8]) -> Result<String, String> {
        if self.config.dry_run {
            self.state.set_status(DeploymentStatus::Completed);
            return Ok("0x0000000000000000000000000000000000000000".to_string());
        }

        self.state.set_status(DeploymentStatus::Confirming);

        let contract_address = generate_address();
        self.state.set_contract_address(&contract_address);

        let tx_hash = generate_tx_hash();
        self.state.set_transaction_hash(&tx_hash);

        self.state.set_block_number(1);
        self.state.set_gas_used(self.config.gas_limit / 2);

        self.state.complete();
        Ok(contract_address)
    }

    pub fn cancel(&mut self) {
        self.state.set_status(DeploymentStatus::Cancelled);
    }

    pub fn estimate_gas(&self, bytecode: &[u8]) -> u64 {
        let base_cost = 53000u64;
        let bytecode_cost = (bytecode.len() as u64) * 200u64;
        base_cost + bytecode_cost
    }
}

fn generate_address() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("0x{:040x}", timestamp)
}

fn generate_tx_hash() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("0x{:064x}", timestamp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deployment::Environment;

    fn create_test_config() -> DeploymentConfig {
        DeploymentConfig::new(Environment::Development)
            .with_gas_limit(5_000_000)
            .with_dry_run(true)
    }

    #[test]
    fn test_deployer_creation() {
        let config = create_test_config();
        let deployer = ContractDeployer::new(config.clone());
        assert_eq!(deployer.get_config().environment, Environment::Development);
        assert_eq!(deployer.get_state().status, DeploymentStatus::Pending);
    }

    #[test]
    fn test_validate_success() {
        let config = create_test_config();
        let mut deployer = ContractDeployer::new(config);
        assert!(deployer.validate().is_ok());
    }

    #[test]
    fn test_validate_zero_gas_limit() {
        let config = DeploymentConfig::new(Environment::Development).with_gas_limit(0);
        let mut deployer = ContractDeployer::new(config);
        assert!(deployer.validate().is_err());
    }

    #[test]
    fn test_validate_zero_timeout() {
        let mut config = DeploymentConfig::new(Environment::Development);
        config.timeout_seconds = 0;
        let mut deployer = ContractDeployer::new(config);
        let result = deployer.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_prepare_dry_run() {
        let config = create_test_config();
        let mut deployer = ContractDeployer::new(config);
        assert_eq!(deployer.prepare().unwrap(), "DRY_RUN");
    }

    #[test]
    fn test_deploy_dry_run() {
        let config = create_test_config();
        let mut deployer = ContractDeployer::new(config);
        deployer.prepare().unwrap();
        let result = deployer.deploy(&[0u8; 100]);
        assert!(result.is_ok());
        assert_eq!(deployer.get_state().status, DeploymentStatus::Completed);
    }

    #[test]
    fn test_estimate_gas() {
        let config = create_test_config();
        let deployer = ContractDeployer::new(config);
        let gas = deployer.estimate_gas(&[0u8; 100]);
        assert_eq!(gas, 53000u64 + 20000u64);
    }

    #[test]
    fn test_cancel() {
        let config = create_test_config();
        let mut deployer = ContractDeployer::new(config);
        deployer.cancel();
        assert_eq!(deployer.get_state().status, DeploymentStatus::Cancelled);
    }
}
