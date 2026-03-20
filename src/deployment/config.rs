//! Deployment Configuration
//! Issue #23 - Contract Deployment and Management Tools

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Deployment environment
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Environment {
    #[default]
    Development,
    Testnet,
    Mainnet,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Development => "development",
            Environment::Testnet => "testnet",
            Environment::Mainnet => "mainnet",
        }
    }
}

/// Network configuration for deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub name: String,
    pub rpc_url: String,
    pub chain_id: u64,
    pub explorer_url: Option<String>,
}

impl NetworkConfig {
    pub fn new(name: &str, rpc_url: &str, chain_id: u64) -> Self {
        NetworkConfig {
            name: name.to_string(),
            rpc_url: rpc_url.to_string(),
            chain_id,
            explorer_url: None,
        }
    }

    pub fn with_explorer(mut self, url: &str) -> Self {
        self.explorer_url = Some(url.to_string());
        self
    }
}

/// Deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    pub environment: Environment,
    pub network: NetworkConfig,
    pub gas_limit: u64,
    pub max_gas_price: Option<u128>,
    pub timeout_seconds: u64,
    pub dry_run: bool,
    pub verify_source: bool,
    pub metadata: HashMap<String, String>,
}

impl DeploymentConfig {
    pub fn new(environment: Environment) -> Self {
        let network = match environment {
            Environment::Development => NetworkConfig::new("local", "http://localhost:9933", 0),
            Environment::Testnet => {
                NetworkConfig::new("testnet", "https://rpc.testnet.polkadot.io", 42)
            }
            Environment::Mainnet => NetworkConfig::new("mainnet", "https://rpc.polkadot.io", 0),
        };

        DeploymentConfig {
            environment,
            network,
            gas_limit: 5_000_000,
            max_gas_price: None,
            timeout_seconds: 300,
            dry_run: false,
            verify_source: true,
            metadata: HashMap::new(),
        }
    }

    pub fn with_gas_limit(mut self, limit: u64) -> Self {
        self.gas_limit = limit;
        self
    }

    pub fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_default() {
        assert_eq!(Environment::default(), Environment::Development);
    }

    #[test]
    fn test_environment_as_str() {
        assert_eq!(Environment::Development.as_str(), "development");
        assert_eq!(Environment::Testnet.as_str(), "testnet");
        assert_eq!(Environment::Mainnet.as_str(), "mainnet");
    }

    #[test]
    fn test_network_config_creation() {
        let network = NetworkConfig::new("local", "http://localhost:9933", 0);
        assert_eq!(network.name, "local");
        assert_eq!(network.chain_id, 0);
        assert!(network.explorer_url.is_none());
    }

    #[test]
    fn test_network_config_with_explorer() {
        let network = NetworkConfig::new("testnet", "https://rpc.testnet.io", 42)
            .with_explorer("https://explorer.testnet.io");
        assert!(network.explorer_url.is_some());
        assert_eq!(network.explorer_url.unwrap(), "https://explorer.testnet.io");
    }

    #[test]
    fn test_deployment_config_creation() {
        let config = DeploymentConfig::new(Environment::Development);
        assert_eq!(config.environment, Environment::Development);
        assert_eq!(config.gas_limit, 5_000_000);
        assert!(!config.dry_run);
        assert!(config.verify_source);
    }

    #[test]
    fn test_deployment_config_with_gas_limit() {
        let config = DeploymentConfig::new(Environment::Mainnet).with_gas_limit(10_000_000);
        assert_eq!(config.gas_limit, 10_000_000);
    }

    #[test]
    fn test_deployment_config_with_dry_run() {
        let config = DeploymentConfig::new(Environment::Testnet).with_dry_run(true);
        assert!(config.dry_run);
    }

    #[test]
    fn test_deployment_config_with_metadata() {
        let config = DeploymentConfig::new(Environment::Development)
            .with_metadata("version", "1.0.0")
            .with_metadata("author", "test");
        assert_eq!(config.metadata.get("version"), Some(&"1.0.0".to_string()));
        assert_eq!(config.metadata.get("author"), Some(&"test".to_string()));
    }
}
