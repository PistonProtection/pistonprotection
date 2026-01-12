//! Permission model definitions for RBAC

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashSet;

/// Permission model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Permission {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub resource_type: String,
    pub action: String,
    pub created_at: DateTime<Utc>,
}

/// Role-Permission mapping
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RolePermission {
    pub id: String,
    pub role_id: String,
    pub permission_id: String,
    pub created_at: DateTime<Utc>,
}

/// Permission check request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionCheck {
    pub user_id: String,
    pub organization_id: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub action: String,
}

/// Permission check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionCheckResult {
    pub allowed: bool,
    pub reason: Option<String>,
}

/// Resource types in the system
pub struct ResourceTypes;

impl ResourceTypes {
    pub const ORGANIZATION: &'static str = "organization";
    pub const MEMBERS: &'static str = "members";
    pub const BACKENDS: &'static str = "backends";
    pub const DOMAINS: &'static str = "domains";
    pub const FILTERS: &'static str = "filters";
    pub const API_KEYS: &'static str = "api_keys";
    pub const AUDIT_LOGS: &'static str = "audit_logs";
    pub const INVITATIONS: &'static str = "invitations";
    pub const ROLES: &'static str = "roles";
    pub const SETTINGS: &'static str = "settings";
    pub const BILLING: &'static str = "billing";

    pub fn all() -> Vec<&'static str> {
        vec![
            Self::ORGANIZATION,
            Self::MEMBERS,
            Self::BACKENDS,
            Self::DOMAINS,
            Self::FILTERS,
            Self::API_KEYS,
            Self::AUDIT_LOGS,
            Self::INVITATIONS,
            Self::ROLES,
            Self::SETTINGS,
            Self::BILLING,
        ]
    }
}

/// Actions that can be performed on resources
pub struct Actions;

impl Actions {
    pub const READ: &'static str = "read";
    pub const CREATE: &'static str = "create";
    pub const UPDATE: &'static str = "update";
    pub const DELETE: &'static str = "delete";
    pub const ADMIN: &'static str = "admin";

    pub fn all() -> Vec<&'static str> {
        vec![Self::READ, Self::CREATE, Self::UPDATE, Self::DELETE, Self::ADMIN]
    }
}

/// Permission helper for checking permissions
pub struct PermissionHelper;

impl PermissionHelper {
    /// Format a permission string
    pub fn format(resource: &str, action: &str) -> String {
        format!("{}:{}", resource, action)
    }

    /// Format a permission with optional scope
    pub fn format_scoped(resource: &str, action: &str, scope: Option<&str>) -> String {
        match scope {
            Some(s) => format!("{}:{}:{}", resource, action, s),
            None => Self::format(resource, action),
        }
    }

    /// Check if a user's permissions allow an action
    pub fn check_permission(
        user_permissions: &HashSet<String>,
        resource: &str,
        action: &str,
    ) -> bool {
        // Check for wildcard permission
        if user_permissions.contains("*") {
            return true;
        }

        // Check for resource wildcard (e.g., "backends:*")
        let resource_wildcard = format!("{}:*", resource);
        if user_permissions.contains(&resource_wildcard) {
            return true;
        }

        // Check for specific permission
        let specific = Self::format(resource, action);
        user_permissions.contains(&specific)
    }

    /// Check if permission allows access with optional scope
    pub fn check_scoped_permission(
        user_permissions: &HashSet<String>,
        resource: &str,
        action: &str,
        scope: Option<&str>,
    ) -> bool {
        // First check unscoped permission
        if Self::check_permission(user_permissions, resource, action) {
            return true;
        }

        // Check scoped permission if scope provided
        if let Some(s) = scope {
            let scoped = Self::format_scoped(resource, action, Some(s));
            if user_permissions.contains(&scoped) {
                return true;
            }
        }

        false
    }

    /// Parse a permission string into (resource, action, scope)
    pub fn parse(permission: &str) -> Option<(String, String, Option<String>)> {
        let parts: Vec<&str> = permission.split(':').collect();
        match parts.len() {
            2 => Some((parts[0].to_string(), parts[1].to_string(), None)),
            3 => Some((
                parts[0].to_string(),
                parts[1].to_string(),
                Some(parts[2].to_string()),
            )),
            _ => None,
        }
    }

    /// Get all standard permissions for a resource
    pub fn all_for_resource(resource: &str) -> Vec<String> {
        Actions::all()
            .iter()
            .map(|action| Self::format(resource, action))
            .collect()
    }

    /// Generate all system permissions
    pub fn all_system_permissions() -> Vec<String> {
        let mut permissions = Vec::new();
        for resource in ResourceTypes::all() {
            for action in Actions::all() {
                permissions.push(Self::format(resource, action));
            }
        }
        // Add wildcard
        permissions.push("*".to_string());
        permissions
    }
}

/// Default permission sets for common use cases
pub struct DefaultPermissions;

impl DefaultPermissions {
    /// Read-only permissions for a resource
    pub fn read_only(resource: &str) -> Vec<String> {
        vec![PermissionHelper::format(resource, Actions::READ)]
    }

    /// Full CRUD permissions for a resource
    pub fn full_crud(resource: &str) -> Vec<String> {
        vec![
            PermissionHelper::format(resource, Actions::CREATE),
            PermissionHelper::format(resource, Actions::READ),
            PermissionHelper::format(resource, Actions::UPDATE),
            PermissionHelper::format(resource, Actions::DELETE),
        ]
    }

    /// Full permissions including admin
    pub fn full_access(resource: &str) -> Vec<String> {
        vec![format!("{}:*", resource)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_format() {
        assert_eq!(
            PermissionHelper::format("backends", "read"),
            "backends:read"
        );
        assert_eq!(
            PermissionHelper::format_scoped("api_keys", "read", Some("own")),
            "api_keys:read:own"
        );
    }

    #[test]
    fn test_permission_check() {
        let mut perms = HashSet::new();
        perms.insert("backends:read".to_string());
        perms.insert("backends:create".to_string());
        perms.insert("domains:*".to_string());

        assert!(PermissionHelper::check_permission(&perms, "backends", "read"));
        assert!(PermissionHelper::check_permission(&perms, "backends", "create"));
        assert!(!PermissionHelper::check_permission(&perms, "backends", "delete"));

        // Wildcard for domains
        assert!(PermissionHelper::check_permission(&perms, "domains", "read"));
        assert!(PermissionHelper::check_permission(&perms, "domains", "delete"));
    }

    #[test]
    fn test_wildcard_permission() {
        let mut perms = HashSet::new();
        perms.insert("*".to_string());

        assert!(PermissionHelper::check_permission(&perms, "anything", "anything"));
    }

    #[test]
    fn test_parse_permission() {
        let (resource, action, scope) = PermissionHelper::parse("backends:read").unwrap();
        assert_eq!(resource, "backends");
        assert_eq!(action, "read");
        assert!(scope.is_none());

        let (resource, action, scope) = PermissionHelper::parse("api_keys:read:own").unwrap();
        assert_eq!(resource, "api_keys");
        assert_eq!(action, "read");
        assert_eq!(scope.unwrap(), "own");
    }
}
