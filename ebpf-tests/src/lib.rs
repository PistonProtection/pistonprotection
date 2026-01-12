//! PistonProtection eBPF/XDP Packet Filter Test Library
//!
//! This library provides packet generation utilities and test helpers
//! for testing XDP packet filters in userspace.

pub mod packet_generator;

// Re-export commonly used items
pub use packet_generator::*;
