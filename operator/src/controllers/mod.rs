//! Controller implementations for PistonProtection CRDs
//!
//! This module contains the reconciliation logic for all custom resources:
//! - DDoSProtection: Main protection configuration
//! - FilterRule: Custom filtering rules
//! - Backend: Backend service definitions (optional)

pub mod backend;
pub mod ddos_protection;
pub mod filter_rule;

// Re-export for convenience
pub use backend::Context as BackendContext;
pub use ddos_protection::Context as DDoSProtectionContext;
pub use filter_rule::Context as FilterRuleContext;
