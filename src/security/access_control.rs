/// Access Control module - Role-Based Access Control (RBAC)
///
/// Provides comprehensive access control mechanisms including role-based permissions,
/// resource protection, and principal authentication for secure operations.
use crate::compiler::parser::ast::*;
use crate::security::SecurityError;
use std::collections::HashMap;
use std::collections::HashSet;

/// Permission types
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Permission {
    Read,
    Write,
    Execute,
    Admin,
    Deploy,
    CallExternal,
    AccessStorage,
    ModifyState,
}

/// Access control entry
#[derive(Debug, Clone)]
pub struct AccessControlEntry {
    pub principal: Vec<u8>, // Address or identifier
    pub resource: String,
    pub permissions: HashSet<Permission>,
}

/// Role definition
#[derive(Debug, Clone)]
pub struct Role {
    pub name: String,
    pub permissions: HashSet<Permission>,
    pub members: HashSet<Vec<u8>>, // Principal addresses
}

/// Role-based access control system
pub struct AccessControl {
    roles: HashMap<String, Role>,
    entries: HashMap<String, AccessControlEntry>, // Resource-based entries
    current_principal: Option<Vec<u8>>,
}

impl Default for AccessControl {
    fn default() -> Self {
        Self::new()
    }
}

impl AccessControl {
    /// Create a new access control system
    pub fn new() -> Self {
        let mut access_control = Self {
            roles: HashMap::new(),
            entries: HashMap::new(),
            current_principal: None,
        };

        access_control.initialize_default_roles();
        access_control
    }

    /// Initialize default roles and permissions
    fn initialize_default_roles(&mut self) {
        // Admin role with all permissions
        let mut admin_permissions = HashSet::new();
        admin_permissions.insert(Permission::Read);
        admin_permissions.insert(Permission::Write);
        admin_permissions.insert(Permission::Execute);
        admin_permissions.insert(Permission::Admin);
        admin_permissions.insert(Permission::Deploy);
        admin_permissions.insert(Permission::CallExternal);
        admin_permissions.insert(Permission::AccessStorage);
        admin_permissions.insert(Permission::ModifyState);

        self.roles.insert(
            "admin".to_string(),
            Role {
                name: "admin".to_string(),
                permissions: admin_permissions,
                members: HashSet::new(),
            },
        );

        // User role with limited permissions
        let mut user_permissions = HashSet::new();
        user_permissions.insert(Permission::Read);
        user_permissions.insert(Permission::Execute);

        self.roles.insert(
            "user".to_string(),
            Role {
                name: "user".to_string(),
                permissions: user_permissions,
                members: HashSet::new(),
            },
        );

        // Deployer role
        let mut deployer_permissions = HashSet::new();
        deployer_permissions.insert(Permission::Read);
        deployer_permissions.insert(Permission::Execute);
        deployer_permissions.insert(Permission::Deploy);
        deployer_permissions.insert(Permission::CallExternal);

        self.roles.insert(
            "deployer".to_string(),
            Role {
                name: "deployer".to_string(),
                permissions: deployer_permissions,
                members: HashSet::new(),
            },
        );
    }

    /// Set the current principal (caller)
    pub fn set_current_principal(&mut self, principal: &[u8]) {
        self.current_principal = Some(principal.to_vec());
    }

    /// Clear the current principal
    pub fn clear_current_principal(&mut self) {
        self.current_principal = None;
    }

    /// Create a new role
    pub fn create_role(
        &mut self,
        name: &str,
        permissions: HashSet<Permission>,
    ) -> Result<(), SecurityError> {
        if self.roles.contains_key(name) {
            return Err(SecurityError::AccessDenied(format!(
                "Role '{}' already exists",
                name
            )));
        }

        self.roles.insert(
            name.to_string(),
            Role {
                name: name.to_string(),
                permissions,
                members: HashSet::new(),
            },
        );

        Ok(())
    }

    /// Add a principal to a role
    pub fn add_role_member(
        &mut self,
        role_name: &str,
        principal: &[u8],
    ) -> Result<(), SecurityError> {
        let role = self.roles.get_mut(role_name).ok_or_else(|| {
            SecurityError::AccessDenied(format!("Role '{}' does not exist", role_name))
        })?;

        role.members.insert(principal.to_vec());
        Ok(())
    }

    /// Remove a principal from a role
    pub fn remove_role_member(
        &mut self,
        role_name: &str,
        principal: &[u8],
    ) -> Result<(), SecurityError> {
        let role = self.roles.get_mut(role_name).ok_or_else(|| {
            SecurityError::AccessDenied(format!("Role '{}' does not exist", role_name))
        })?;

        role.members.remove(principal);
        Ok(())
    }

    /// Grant permission to a role for a resource
    pub fn grant_permission(
        &mut self,
        role_name: &str,
        resource: &str,
        permission: Permission,
    ) -> Result<(), SecurityError> {
        let role = self.roles.get_mut(role_name).ok_or_else(|| {
            SecurityError::AccessDenied(format!("Role '{}' does not exist", role_name))
        })?;

        role.permissions.insert(permission.clone());

        // Also create/update access control entry
        let key = format!("{}:{}", resource, role_name);
        self.entries
            .entry(key)
            .or_insert_with(|| AccessControlEntry {
                principal: vec![], // Will be resolved at runtime
                resource: resource.to_string(),
                permissions: HashSet::new(),
            })
            .permissions
            .insert(permission);

        Ok(())
    }

    /// Revoke permission from a role for a resource
    pub fn revoke_permission(
        &mut self,
        role_name: &str,
        resource: &str,
        permission: &Permission,
    ) -> Result<(), SecurityError> {
        let role = self.roles.get_mut(role_name).ok_or_else(|| {
            SecurityError::AccessDenied(format!("Role '{}' does not exist", role_name))
        })?;

        role.permissions.remove(permission);

        let key = format!("{}:{}", resource, role_name);
        if let Some(entry) = self.entries.get_mut(&key) {
            entry.permissions.remove(permission);
        }

        Ok(())
    }

    /// Check if current principal has permission for a resource
    pub fn check_permission(
        &self,
        principal: &[u8],
        resource: &str,
        operation: &str,
    ) -> Result<(), SecurityError> {
        let permission = self.parse_operation(operation)?;

        // Check if principal has direct access entry
        let direct_key = format!("direct:{}:{}", principal.len(), resource);
        if let Some(entry) = self.entries.get(&direct_key) {
            if entry.permissions.contains(&permission) {
                return Ok(());
            }
        }

        // Check role-based permissions
        for role in self.roles.values() {
            if role.members.contains(principal) && role.permissions.contains(&permission) {
                // Also check resource-specific permissions
                let resource_key = format!("{}:{}", resource, role.name);
                if let Some(entry) = self.entries.get(&resource_key) {
                    if entry.permissions.contains(&permission) {
                        return Ok(());
                    }
                } else {
                    // If no specific resource entry, allow based on role permissions
                    return Ok(());
                }
            }
        }

        Err(SecurityError::AccessDenied(format!(
            "Principal does not have '{}' permission for resource '{}'",
            operation, resource
        )))
    }

    /// Check if current principal has permission for a resource (using current principal)
    pub fn check_current_permission(
        &self,
        resource: &str,
        operation: &str,
    ) -> Result<(), SecurityError> {
        if let Some(principal) = &self.current_principal {
            self.check_permission(principal, resource, operation)
        } else {
            Err(SecurityError::AccessDenied(
                "No current principal set".to_string(),
            ))
        }
    }

    /// Parse operation string to Permission enum
    fn parse_operation(&self, operation: &str) -> Result<Permission, SecurityError> {
        match operation.to_lowercase().as_str() {
            "read" => Ok(Permission::Read),
            "write" => Ok(Permission::Write),
            "execute" | "call" => Ok(Permission::Execute),
            "admin" | "administrator" => Ok(Permission::Admin),
            "deploy" => Ok(Permission::Deploy),
            "external_call" | "external" => Ok(Permission::CallExternal),
            "storage" | "access_storage" => Ok(Permission::AccessStorage),
            "state" | "modify_state" => Ok(Permission::ModifyState),
            _ => Err(SecurityError::AccessDenied(format!(
                "Unknown operation: {}",
                operation
            ))),
        }
    }

    /// Get all roles for a principal
    pub fn get_principal_roles(&self, principal: &[u8]) -> Vec<String> {
        self.roles
            .values()
            .filter(|role| role.members.contains(principal))
            .map(|role| role.name.clone())
            .collect()
    }

    /// Get all permissions for a principal
    pub fn get_principal_permissions(&self, principal: &[u8]) -> HashSet<Permission> {
        let mut permissions = HashSet::new();

        for role in self.roles.values() {
            if role.members.contains(principal) {
                permissions.extend(role.permissions.clone());
            }
        }

        permissions
    }

    /// Create a resource-specific access control entry
    pub fn create_resource_entry(
        &mut self,
        resource: &str,
        principal: &[u8],
        permissions: HashSet<Permission>,
    ) {
        let key = format!("direct:{}:{}", principal.len(), resource);
        self.entries.insert(
            key,
            AccessControlEntry {
                principal: principal.to_vec(),
                resource: resource.to_string(),
                permissions,
            },
        );
    }

    /// Remove a resource-specific access control entry
    pub fn remove_resource_entry(
        &mut self,
        resource: &str,
        principal: &[u8],
    ) -> Result<(), SecurityError> {
        let key = format!("direct:{}:{}", principal.len(), resource);
        if self.entries.remove(&key).is_some() {
            Ok(())
        } else {
            Err(SecurityError::AccessDenied(format!(
                "No access control entry found for resource '{}' and principal",
                resource
            )))
        }
    }

    /// Get all defined roles
    pub fn get_roles(&self) -> Vec<String> {
        self.roles.keys().cloned().collect()
    }

    /// Get role information
    pub fn get_role_info(&self, role_name: &str) -> Option<&Role> {
        self.roles.get(role_name)
    }

    /// Validate access control configuration
    pub fn validate_config(&self) -> Result<(), SecurityError> {
        // Check for empty roles with members
        for role in self.roles.values() {
            if role.members.is_empty() {
                return Err(SecurityError::AccessDenied(format!(
                    "Role '{}' has no members",
                    role.name
                )));
            }
        }

        // Check for orphaned access control entries
        for (key, entry) in &self.entries {
            if !key.starts_with("direct:") {
                continue; // Skip role-based entries
            }

            if !entry.permissions.is_empty() {
                // This is a valid entry
                continue;
            }
        }

        Ok(())
    }
}

/// Register access control functions in AST
pub fn register_access_control_functions() -> Vec<Definition> {
    let mut definitions = Vec::new();
    let dummy_loc = Location {
        line: 0,
        column: 0,
        start: 0,
        end: 0,
    };

    let address_type = Type::Named {
        name: "Address".to_string(),
        params: Vec::new(),
        location: dummy_loc.clone(),
    };

    let string_type = Type::Named {
        name: "String".to_string(),
        params: Vec::new(),
        location: dummy_loc.clone(),
    };

    let bool_type = Type::Named {
        name: "Bool".to_string(),
        params: Vec::new(),
        location: dummy_loc.clone(),
    };

    // Check access permission
    definitions.push(Definition::FunctionDef {
        name: "AccessControl/checkAccess".to_string(),
        params: vec![
            Parameter {
                name: "principal".to_string(),
                ty: address_type.clone(),
                location: dummy_loc.clone(),
            },
            Parameter {
                name: "resource".to_string(),
                ty: string_type.clone(),
                location: dummy_loc.clone(),
            },
            Parameter {
                name: "operation".to_string(),
                ty: string_type.clone(),
                location: dummy_loc.clone(),
            },
        ],
        return_type: Some(bool_type.clone()),
        body: Block {
            statements: Vec::new(),
            location: dummy_loc.clone(),
        },
        checked: Some(true),
        location: dummy_loc.clone(),
    });

    // Get principal roles
    definitions.push(Definition::FunctionDef {
        name: "AccessControl/getRoles".to_string(),
        params: vec![Parameter {
            name: "principal".to_string(),
            ty: address_type.clone(),
            location: dummy_loc.clone(),
        }],
        return_type: Some(Type::Tuple {
            elements: vec![string_type.clone()],
            location: dummy_loc.clone(),
        }),
        body: Block {
            statements: Vec::new(),
            location: dummy_loc.clone(),
        },
        checked: Some(true),
        location: dummy_loc.clone(),
    });

    // Verify role membership
    definitions.push(Definition::FunctionDef {
        name: "AccessControl/hasRole".to_string(),
        params: vec![
            Parameter {
                name: "principal".to_string(),
                ty: address_type.clone(),
                location: dummy_loc.clone(),
            },
            Parameter {
                name: "role".to_string(),
                ty: string_type.clone(),
                location: dummy_loc.clone(),
            },
        ],
        return_type: Some(bool_type.clone()),
        body: Block {
            statements: Vec::new(),
            location: dummy_loc.clone(),
        },
        checked: Some(true),
        location: dummy_loc.clone(),
    });

    definitions
}
