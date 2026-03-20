//! Deployment State
//! Issue #23 - Contract Deployment and Management Tools

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Deployment status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DeploymentStatus {
    #[default]
    Pending,
    Compiling,
    Validating,
    Deploying,
    Confirming,
    Completed,
    Failed,
    Cancelled,
}

/// Deployment state tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentState {
    pub status: DeploymentStatus,
    pub contract_address: Option<String>,
    pub transaction_hash: Option<String>,
    pub block_number: Option<u64>,
    pub gas_used: Option<u64>,
    pub error_message: Option<String>,
    pub started_at: u64,
    pub completed_at: Option<u64>,
    pub deployment_config: Option<String>,
}

impl DeploymentState {
    pub fn new() -> Self {
        DeploymentState {
            status: DeploymentStatus::Pending,
            contract_address: None,
            transaction_hash: None,
            block_number: None,
            gas_used: None,
            error_message: None,
            started_at: current_timestamp(),
            completed_at: None,
            deployment_config: None,
        }
    }

    pub fn with_config(mut self, config: &str) -> Self {
        self.deployment_config = Some(config.to_string());
        self
    }

    pub fn set_status(&mut self, status: DeploymentStatus) {
        self.status = status;
    }

    pub fn set_contract_address(&mut self, address: &str) {
        self.contract_address = Some(address.to_string());
    }

    pub fn set_transaction_hash(&mut self, tx_hash: &str) {
        self.transaction_hash = Some(tx_hash.to_string());
    }

    pub fn set_block_number(&mut self, block: u64) {
        self.block_number = Some(block);
    }

    pub fn set_gas_used(&mut self, gas: u64) {
        self.gas_used = Some(gas);
    }

    pub fn set_error(&mut self, error: &str) {
        self.error_message = Some(error.to_string());
        self.status = DeploymentStatus::Failed;
        self.completed_at = Some(current_timestamp());
    }

    pub fn complete(&mut self) {
        self.status = DeploymentStatus::Completed;
        self.completed_at = Some(current_timestamp());
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            DeploymentStatus::Completed | DeploymentStatus::Failed | DeploymentStatus::Cancelled
        )
    }

    pub fn duration_seconds(&self) -> Option<u64> {
        self.completed_at
            .map(|completed| completed - self.started_at)
    }
}

impl Default for DeploymentState {
    fn default() -> Self {
        Self::new()
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deployment_state_creation() {
        let state = DeploymentState::new();
        assert_eq!(state.status, DeploymentStatus::Pending);
        assert!(state.contract_address.is_none());
        assert!(state.transaction_hash.is_none());
        assert!(state.gas_used.is_none());
    }

    #[test]
    fn test_deployment_state_with_config() {
        let state = DeploymentState::new().with_config("test_config");
        assert_eq!(state.deployment_config, Some("test_config".to_string()));
    }

    #[test]
    fn test_set_status() {
        let mut state = DeploymentState::new();
        state.set_status(DeploymentStatus::Deploying);
        assert_eq!(state.status, DeploymentStatus::Deploying);
    }

    #[test]
    fn test_set_contract_address() {
        let mut state = DeploymentState::new();
        state.set_contract_address("0x123abc");
        assert_eq!(state.contract_address, Some("0x123abc".to_string()));
    }

    #[test]
    fn test_set_error() {
        let mut state = DeploymentState::new();
        state.set_error("Out of gas");
        assert_eq!(state.status, DeploymentStatus::Failed);
        assert_eq!(state.error_message, Some("Out of gas".to_string()));
        assert!(state.completed_at.is_some());
    }

    #[test]
    fn test_complete() {
        let mut state = DeploymentState::new();
        state.complete();
        assert_eq!(state.status, DeploymentStatus::Completed);
        assert!(state.completed_at.is_some());
    }

    #[test]
    fn test_is_terminal() {
        let mut state = DeploymentState::new();
        assert!(!state.is_terminal());

        state.status = DeploymentStatus::Completed;
        assert!(state.is_terminal());

        state.status = DeploymentStatus::Failed;
        assert!(state.is_terminal());

        state.status = DeploymentStatus::Cancelled;
        assert!(state.is_terminal());

        state.status = DeploymentStatus::Pending;
        assert!(!state.is_terminal());
    }
}
