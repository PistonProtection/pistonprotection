//! eBPF XDP Packet Filter Test Suite
//!
//! This module provides comprehensive testing for the XDP packet filters.
//! Tests run in userspace using mock packet data to verify filter logic.

// Use the library crate for packet generation
use pistonprotection_ebpf_tests::packet_generator;

mod varint_tests;
mod minecraft_tests;
mod tcp_tests;
mod raknet_tests;
mod http_tests;

/// Test configuration defaults
pub const DEFAULT_TEST_TIMEOUT_MS: u64 = 1000;

/// Mock XDP action constants (matching eBPF bindings)
pub mod xdp_action {
    pub const XDP_ABORTED: u32 = 0;
    pub const XDP_DROP: u32 = 1;
    pub const XDP_PASS: u32 = 2;
    pub const XDP_TX: u32 = 3;
    pub const XDP_REDIRECT: u32 = 4;
}

/// Test result tracking
#[derive(Debug, Default)]
pub struct TestStats {
    pub passed: u32,
    pub failed: u32,
    pub skipped: u32,
}

impl TestStats {
    pub fn record_pass(&mut self) {
        self.passed += 1;
    }

    pub fn record_fail(&mut self) {
        self.failed += 1;
    }

    pub fn record_skip(&mut self) {
        self.skipped += 1;
    }

    pub fn total(&self) -> u32 {
        self.passed + self.failed + self.skipped
    }

    pub fn success_rate(&self) -> f64 {
        if self.total() == 0 {
            0.0
        } else {
            (self.passed as f64) / (self.total() as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_stats_tracking() {
        let mut stats = TestStats::default();
        stats.record_pass();
        stats.record_pass();
        stats.record_fail();
        stats.record_skip();

        assert_eq!(stats.passed, 2);
        assert_eq!(stats.failed, 1);
        assert_eq!(stats.skipped, 1);
        assert_eq!(stats.total(), 4);
        assert!((stats.success_rate() - 50.0).abs() < 0.001);
    }
}
