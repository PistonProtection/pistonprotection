//! Audit logging service

use sqlx::PgPool;
use tracing::debug;

use crate::db;
use crate::models::{AuditLogBuilder, AuditLogEntry, AuditLogFilter, CreateAuditLogRequest};

/// Audit service for logging actions
pub struct AuditService {
    db: PgPool,
}

impl AuditService {
    /// Create a new audit service
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Create an audit log entry
    pub async fn log(&self, request: CreateAuditLogRequest) -> Result<AuditLogEntry, AuditError> {
        let id = uuid::Uuid::new_v4().to_string();

        let entry = db::create_audit_log(
            &self.db,
            &id,
            &request.organization_id,
            request.user_id.as_deref(),
            request.user_email.as_deref(),
            &request.action,
            &request.resource_type,
            request.resource_id.as_deref(),
            &request.description,
            &request.metadata,
            request.ip_address.as_deref(),
            request.user_agent.as_deref(),
        )
        .await
        .map_err(|e| AuditError::DatabaseError(e.to_string()))?;

        debug!(
            "Audit log created: action={}, resource={}, user={:?}",
            entry.action, entry.resource_type, entry.user_id
        );

        Ok(entry)
    }

    /// Log using builder pattern
    pub async fn log_builder(&self, builder: AuditLogBuilder) -> Result<AuditLogEntry, AuditError> {
        self.log(builder.build()).await
    }

    /// List audit logs with filters
    pub async fn list(
        &self,
        filter: &AuditLogFilter,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<AuditLogEntry>, u32), AuditError> {
        db::list_audit_logs(&self.db, filter, page, page_size)
            .await
            .map_err(|e| AuditError::DatabaseError(e.to_string()))
    }

    /// Helper methods for common audit actions

    /// Log user login
    pub async fn log_login(
        &self,
        org_id: &str,
        user_id: &str,
        email: &str,
        ip: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<AuditLogEntry, AuditError> {
        let builder = AuditLogBuilder::new(org_id, "user.login", "user")
            .user(user_id, Some(email))
            .description("User logged in")
            .request_info(ip, user_agent);

        self.log_builder(builder).await
    }

    /// Log user logout
    pub async fn log_logout(
        &self,
        org_id: &str,
        user_id: &str,
        email: &str,
        ip: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<AuditLogEntry, AuditError> {
        let builder = AuditLogBuilder::new(org_id, "user.logout", "user")
            .user(user_id, Some(email))
            .description("User logged out")
            .request_info(ip, user_agent);

        self.log_builder(builder).await
    }

    /// Log resource creation
    pub async fn log_create(
        &self,
        org_id: &str,
        user_id: &str,
        email: Option<&str>,
        resource_type: &str,
        resource_id: &str,
        description: &str,
        ip: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<AuditLogEntry, AuditError> {
        let action = format!("{}.created", resource_type);
        let builder = AuditLogBuilder::new(org_id, &action, resource_type)
            .user(user_id, email)
            .resource(resource_id)
            .description(description)
            .request_info(ip, user_agent);

        self.log_builder(builder).await
    }

    /// Log resource update
    pub async fn log_update(
        &self,
        org_id: &str,
        user_id: &str,
        email: Option<&str>,
        resource_type: &str,
        resource_id: &str,
        description: &str,
        ip: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<AuditLogEntry, AuditError> {
        let action = format!("{}.updated", resource_type);
        let builder = AuditLogBuilder::new(org_id, &action, resource_type)
            .user(user_id, email)
            .resource(resource_id)
            .description(description)
            .request_info(ip, user_agent);

        self.log_builder(builder).await
    }

    /// Log resource deletion
    pub async fn log_delete(
        &self,
        org_id: &str,
        user_id: &str,
        email: Option<&str>,
        resource_type: &str,
        resource_id: &str,
        description: &str,
        ip: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<AuditLogEntry, AuditError> {
        let action = format!("{}.deleted", resource_type);
        let builder = AuditLogBuilder::new(org_id, &action, resource_type)
            .user(user_id, email)
            .resource(resource_id)
            .description(description)
            .request_info(ip, user_agent);

        self.log_builder(builder).await
    }

    /// Log member action
    pub async fn log_member_action(
        &self,
        org_id: &str,
        actor_id: &str,
        actor_email: Option<&str>,
        action: &str, // "added", "removed", "role_changed"
        target_user_id: &str,
        description: &str,
        ip: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<AuditLogEntry, AuditError> {
        let action_str = format!("member.{}", action);
        let builder = AuditLogBuilder::new(org_id, &action_str, "member")
            .user(actor_id, actor_email)
            .resource(target_user_id)
            .description(description)
            .request_info(ip, user_agent);

        self.log_builder(builder).await
    }

    /// Log API key action
    pub async fn log_api_key_action(
        &self,
        org_id: &str,
        user_id: &str,
        email: Option<&str>,
        action: &str, // "created", "revoked", "used"
        key_id: &str,
        key_name: &str,
        ip: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<AuditLogEntry, AuditError> {
        let action_str = format!("api_key.{}", action);
        let description = format!("API key '{}' {}", key_name, action);
        let builder = AuditLogBuilder::new(org_id, &action_str, "api_key")
            .user(user_id, email)
            .resource(key_id)
            .description(&description)
            .metadata("key_name", key_name)
            .request_info(ip, user_agent);

        self.log_builder(builder).await
    }
}

/// Audit service errors
#[derive(Debug, thiserror::Error)]
pub enum AuditError {
    #[error("Database error: {0}")]
    DatabaseError(String),
}

impl From<AuditError> for tonic::Status {
    fn from(err: AuditError) -> Self {
        match err {
            AuditError::DatabaseError(msg) => tonic::Status::internal(msg),
        }
    }
}
