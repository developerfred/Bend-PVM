use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct VersionInfo {
    pub version: u32,
    pub implementation: String,
    pub deployed_at: u64,
}

#[derive(Debug, Clone)]
pub struct UpgradeRecord {
    pub from_version: u32,
    pub to_version: u32,
    pub new_implementation: String,
}

#[derive(Debug, Clone)]
pub struct ProxyState {
    implementation: String,
    admin: Option<String>,
    version: u32,
    is_paused: bool,
    upgrade_history: VecDeque<UpgradeRecord>,
    deployed_at: u64,
}

impl ProxyState {
    pub fn new(implementation: String) -> Self {
        ProxyState {
            implementation,
            admin: None,
            version: 1,
            is_paused: false,
            upgrade_history: VecDeque::new(),
            deployed_at: 0,
        }
    }

    pub fn implementation(&self) -> &str {
        &self.implementation
    }

    pub fn version(&self) -> u32 {
        self.version
    }

    pub fn admin(&self) -> Option<&String> {
        self.admin.as_ref()
    }

    pub fn is_paused(&self) -> bool {
        self.is_paused
    }

    pub fn upgrade_history(&self) -> Vec<UpgradeRecord> {
        self.upgrade_history.iter().cloned().collect()
    }

    pub fn deployed_at(&self) -> u64 {
        self.deployed_at
    }

    pub fn set_admin(&mut self, admin: String) -> Result<(), ProxyError> {
        if admin.is_empty() {
            return Err(ProxyError::InvalidAdmin);
        }
        self.admin = Some(admin);
        Ok(())
    }

    pub fn upgrade(&mut self, new_implementation: String) -> Result<(), ProxyError> {
        if self.admin.is_none() {
            return Err(ProxyError::Unauthorized);
        }
        if self.is_paused {
            return Err(ProxyError::ContractPaused);
        }
        if new_implementation.is_empty() {
            return Err(ProxyError::InvalidImplementation);
        }

        let old_version = self.version;
        self.version += 1;
        self.implementation = new_implementation;

        self.upgrade_history.push_back(UpgradeRecord {
            from_version: old_version,
            to_version: self.version,
            new_implementation: self.implementation.clone(),
        });

        Ok(())
    }

    pub fn pause(&mut self) -> Result<(), ProxyError> {
        if self.admin.is_none() {
            return Err(ProxyError::Unauthorized);
        }
        self.is_paused = true;
        Ok(())
    }

    pub fn unpause(&mut self) -> Result<(), ProxyError> {
        if self.admin.is_none() {
            return Err(ProxyError::Unauthorized);
        }
        self.is_paused = false;
        Ok(())
    }

    pub fn change_admin(&mut self, new_admin: String) -> Result<(), ProxyError> {
        if self.admin.is_none() {
            return Err(ProxyError::Unauthorized);
        }
        if new_admin.is_empty() {
            return Err(ProxyError::InvalidAdmin);
        }
        self.admin = Some(new_admin);
        Ok(())
    }

    pub fn forward_call(
        &self,
        _function: &str,
        _params: Vec<String>,
    ) -> Result<String, ProxyError> {
        if self.is_paused {
            return Err(ProxyError::ContractPaused);
        }
        Ok(format!("forwarded to {}", self.implementation))
    }

    pub fn clone_with_version(&self, version: u32) -> Self {
        let mut new_proxy = self.clone();
        new_proxy.version = version;
        new_proxy
    }
}

pub trait Proxy {
    fn get_implementation(&self) -> String;
    fn get_admin(&self) -> Option<String>;
    fn get_version(&self) -> u32;
}

impl Proxy for ProxyState {
    fn get_implementation(&self) -> String {
        self.implementation.clone()
    }

    fn get_admin(&self) -> Option<String> {
        self.admin.clone()
    }

    fn get_version(&self) -> u32 {
        self.version
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProxyError {
    Unauthorized,
    ContractPaused,
    InvalidAdmin,
    InvalidImplementation,
    ImplementationNotFound,
}

impl std::fmt::Display for ProxyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProxyError::Unauthorized => write!(f, "Unauthorized: admin access required"),
            ProxyError::ContractPaused => write!(f, "Contract is paused"),
            ProxyError::InvalidAdmin => write!(f, "Invalid admin address"),
            ProxyError::InvalidImplementation => write!(f, "Invalid implementation address"),
            ProxyError::ImplementationNotFound => write!(f, "Implementation not found"),
        }
    }
}

impl std::error::Error for ProxyError {}
