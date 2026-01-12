//! Role model definitions for RBAC

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashSet;
use validator::Validate;

/// Role model for custom RBAC roles
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Role {
    pub id: String,
    pub organization_id: Option<String>, // None for system roles
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Role with its permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleWithPermissions {
    pub role: Role,
    pub permissions: Vec<String>,
}

/// Request to create a new role
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateRoleRequest {
    #[validate(length(min = 1, max = 50, message = "Name must be 1-50 characters"))]
    pub name: String,

    #[validate(length(min = 1, max = 100, message = "Display name must be 1-100 characters"))]
    pub display_name: String,

    #[validate(length(max = 500, message = "Description must be max 500 characters"))]
    pub description: Option<String>,

    pub permissions: Vec<String>,
}

/// Request to update a role
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpdateRoleRequest {
    #[validate(length(min = 1, max = 100, message = "Display name must be 1-100 characters"))]
    pub display_name: Option<String>,

    #[validate(length(max = 500, message = "Description must be max 500 characters"))]
    pub description: Option<String>,

    pub permissions: Option<Vec<String>>,
}

/// Role assignment to a user
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RoleAssignment {
    pub id: String,
    pub user_id: String,
    pub role_id: String,
    pub organization_id: String,
    pub created_at: DateTime<Utc>,
}

/// System-defined roles
pub struct SystemRoles;

impl SystemRoles {
    pub const OWNER: &'static str = "owner";
    pub const ADMIN: &'static str = "admin";
    pub const MEMBER: &'static str = "member";
    pub const VIEWER: &'static str = "viewer";

    /// Get default permissions for owner role
    pub fn owner_permissions() -> HashSet<String> {
        let mut perms = HashSet::new();
        // Full access to everything
        perms.insert("*".to_string());
        perms
    }

    /// Get default permissions for admin role
    pub fn admin_permissions() -> HashSet<String> {
        let mut perms = HashSet::new();
        // Organization management
        perms.insert("organization:read".to_string());
        perms.insert("organization:update".to_string());
        // Member management
        perms.insert("members:read".to_string());
        perms.insert("members:create".to_string());
        perms.insert("members:update".to_string());
        perms.insert("members:delete".to_string());
        // Resource management
        perms.insert("backends:*".to_string());
        perms.insert("domains:*".to_string());
        perms.insert("filters:*".to_string());
        // API keys
        perms.insert("api_keys:read".to_string());
        perms.insert("api_keys:create".to_string());
        perms.insert("api_keys:delete".to_string());
        // Audit logs
        perms.insert("audit_logs:read".to_string());
        // Invitations
        perms.insert("invitations:*".to_string());
        perms
    }

    /// Get default permissions for member role
    pub fn member_permissions() -> HashSet<String> {
        let mut perms = HashSet::new();
        // Organization read
        perms.insert("organization:read".to_string());
        // Member read
        perms.insert("members:read".to_string());
        // Resource management (create, read, update)
        perms.insert("backends:read".to_string());
        perms.insert("backends:create".to_string());
        perms.insert("backends:update".to_string());
        perms.insert("domains:read".to_string());
        perms.insert("domains:create".to_string());
        perms.insert("domains:update".to_string());
        perms.insert("filters:read".to_string());
        perms.insert("filters:create".to_string());
        perms.insert("filters:update".to_string());
        // Own API keys only
        perms.insert("api_keys:read:own".to_string());
        perms.insert("api_keys:create:own".to_string());
        perms.insert("api_keys:delete:own".to_string());
        perms
    }

    /// Get default permissions for viewer role
    pub fn viewer_permissions() -> HashSet<String> {
        let mut perms = HashSet::new();
        // Read-only access
        perms.insert("organization:read".to_string());
        perms.insert("members:read".to_string());
        perms.insert("backends:read".to_string());
        perms.insert("domains:read".to_string());
        perms.insert("filters:read".to_string());
        perms
    }

    /// Get permissions for a system role
    pub fn get_permissions(role_name: &str) -> HashSet<String> {
        match role_name {
            Self::OWNER => Self::owner_permissions(),
            Self::ADMIN => Self::admin_permissions(),
            Self::MEMBER => Self::member_permissions(),
            Self::VIEWER => Self::viewer_permissions(),
            _ => HashSet::new(),
        }
    }

    /// Check if a role name is a system role
    pub fn is_system_role(name: &str) -> bool {
        matches!(
            name,
            Self::OWNER | Self::ADMIN | Self::MEMBER | Self::VIEWER
        )
    }

    /// Get all system role names
    pub fn all() -> Vec<&'static str> {
        vec![Self::OWNER, Self::ADMIN, Self::MEMBER, Self::VIEWER]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_roles() {
        assert!(SystemRoles::is_system_role("owner"));
        assert!(SystemRoles::is_system_role("admin"));
        assert!(!SystemRoles::is_system_role("custom_role"));
    }

    #[test]
    fn test_owner_has_wildcard() {
        let perms = SystemRoles::owner_permissions();
        assert!(perms.contains("*"));
    }

    #[test]
    fn test_admin_permissions() {
        let perms = SystemRoles::admin_permissions();
        assert!(perms.contains("organization:read"));
        assert!(perms.contains("members:create"));
        assert!(!perms.contains("*"));
    }

    #[test]
    fn test_viewer_read_only() {
        let perms = SystemRoles::viewer_permissions();
        for perm in &perms {
            assert!(
                perm.contains(":read") || perm == "*",
                "Viewer should only have read permissions"
            );
        }
    }
}
