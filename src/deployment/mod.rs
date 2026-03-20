//! Deployment Module
//! Issue #23 - Contract Deployment and Management Tools

mod config;
mod deployer;
mod state;

pub use config::{DeploymentConfig, Environment, NetworkConfig};
pub use deployer::ContractDeployer;
pub use state::{DeploymentState, DeploymentStatus};

/// Initialize deployment system with environment
pub fn init_deployment(env: Environment) -> DeploymentConfig {
    DeploymentConfig::new(env)
}
