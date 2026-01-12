//! eBPF map management tests

use super::test_utils::constants;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};

/// Mock map manager for testing without eBPF
struct MockMapManager {
    blocklist: HashMap<u32, u64>, // IP -> timestamp
    allowlist: HashMap<u32, u64>, // IP -> timestamp
    rate_limits: HashMap<u32, RateLimitEntry>,
    connection_stats: HashMap<u32, ConnectionStats>,
}

#[derive(Clone, Debug)]
struct RateLimitEntry {
    tokens: u64,
    last_update: u64,
    limit: u64,
}

#[derive(Clone, Debug, Default)]
struct ConnectionStats {
    packets_total: u64,
    bytes_total: u64,
    packets_dropped: u64,
    bytes_dropped: u64,
}

impl MockMapManager {
    fn new() -> Self {
        Self {
            blocklist: HashMap::new(),
            allowlist: HashMap::new(),
            rate_limits: HashMap::new(),
            connection_stats: HashMap::new(),
        }
    }

    // Blocklist operations
    fn add_to_blocklist(&mut self, ip: Ipv4Addr, expires_at: u64) {
        let ip_u32 = u32::from_be_bytes(ip.octets());
        self.blocklist.insert(ip_u32, expires_at);
    }

    fn remove_from_blocklist(&mut self, ip: Ipv4Addr) -> bool {
        let ip_u32 = u32::from_be_bytes(ip.octets());
        self.blocklist.remove(&ip_u32).is_some()
    }

    fn is_blocked(&self, ip: Ipv4Addr) -> bool {
        let ip_u32 = u32::from_be_bytes(ip.octets());
        self.blocklist.contains_key(&ip_u32)
    }

    fn get_blocklist(&self) -> Vec<Ipv4Addr> {
        self.blocklist
            .keys()
            .map(|&ip| Ipv4Addr::from(ip.to_be_bytes()))
            .collect()
    }

    // Allowlist operations
    fn add_to_allowlist(&mut self, ip: Ipv4Addr, expires_at: u64) {
        let ip_u32 = u32::from_be_bytes(ip.octets());
        self.allowlist.insert(ip_u32, expires_at);
    }

    fn remove_from_allowlist(&mut self, ip: Ipv4Addr) -> bool {
        let ip_u32 = u32::from_be_bytes(ip.octets());
        self.allowlist.remove(&ip_u32).is_some()
    }

    fn is_allowed(&self, ip: Ipv4Addr) -> bool {
        let ip_u32 = u32::from_be_bytes(ip.octets());
        self.allowlist.contains_key(&ip_u32)
    }

    // Rate limit operations
    fn set_rate_limit(&mut self, ip: Ipv4Addr, limit: u64) {
        let ip_u32 = u32::from_be_bytes(ip.octets());
        self.rate_limits.insert(
            ip_u32,
            RateLimitEntry {
                tokens: limit,
                last_update: 0,
                limit,
            },
        );
    }

    fn check_rate_limit(&self, ip: Ipv4Addr) -> bool {
        let ip_u32 = u32::from_be_bytes(ip.octets());
        if let Some(entry) = self.rate_limits.get(&ip_u32) {
            entry.tokens > 0
        } else {
            true // No limit = allowed
        }
    }

    fn consume_token(&mut self, ip: Ipv4Addr) -> bool {
        let ip_u32 = u32::from_be_bytes(ip.octets());
        if let Some(entry) = self.rate_limits.get_mut(&ip_u32) {
            if entry.tokens > 0 {
                entry.tokens -= 1;
                true
            } else {
                false
            }
        } else {
            true
        }
    }

    // Stats operations
    fn increment_stats(
        &mut self,
        ip: Ipv4Addr,
        packets: u64,
        bytes: u64,
        dropped_packets: u64,
        dropped_bytes: u64,
    ) {
        let ip_u32 = u32::from_be_bytes(ip.octets());
        let stats = self.connection_stats.entry(ip_u32).or_default();
        stats.packets_total += packets;
        stats.bytes_total += bytes;
        stats.packets_dropped += dropped_packets;
        stats.bytes_dropped += dropped_bytes;
    }

    fn get_stats(&self, ip: Ipv4Addr) -> Option<&ConnectionStats> {
        let ip_u32 = u32::from_be_bytes(ip.octets());
        self.connection_stats.get(&ip_u32)
    }

    fn clear_stats(&mut self) {
        self.connection_stats.clear();
    }

    // Cleanup operations
    fn cleanup_expired(&mut self, current_time: u64) {
        self.blocklist
            .retain(|_, &mut expires| expires > current_time || expires == 0);
        self.allowlist
            .retain(|_, &mut expires| expires > current_time || expires == 0);
    }
}

// ============================================================================
// Blocklist Tests
// ============================================================================

#[cfg(test)]
mod blocklist_tests {
    use super::*;

    /// Test adding IP to blocklist
    #[test]
    fn test_add_to_blocklist() {
        let mut manager = MockMapManager::new();
        let ip = Ipv4Addr::new(192, 168, 1, 100);

        manager.add_to_blocklist(ip, 0);

        assert!(manager.is_blocked(ip));
    }

    /// Test removing IP from blocklist
    #[test]
    fn test_remove_from_blocklist() {
        let mut manager = MockMapManager::new();
        let ip = Ipv4Addr::new(192, 168, 1, 100);

        manager.add_to_blocklist(ip, 0);
        assert!(manager.is_blocked(ip));

        let removed = manager.remove_from_blocklist(ip);
        assert!(removed);
        assert!(!manager.is_blocked(ip));
    }

    /// Test remove non-existent returns false
    #[test]
    fn test_remove_nonexistent() {
        let mut manager = MockMapManager::new();
        let ip = Ipv4Addr::new(192, 168, 1, 100);

        let removed = manager.remove_from_blocklist(ip);
        assert!(!removed);
    }

    /// Test multiple IPs in blocklist
    #[test]
    fn test_multiple_blocked() {
        let mut manager = MockMapManager::new();

        let ips: Vec<Ipv4Addr> = (1..=10).map(|i| Ipv4Addr::new(192, 168, 1, i)).collect();

        for ip in &ips {
            manager.add_to_blocklist(*ip, 0);
        }

        for ip in &ips {
            assert!(manager.is_blocked(*ip));
        }

        let blocklist = manager.get_blocklist();
        assert_eq!(blocklist.len(), 10);
    }

    /// Test blocklist with expiration
    #[test]
    fn test_blocklist_expiration() {
        let mut manager = MockMapManager::new();
        let ip = Ipv4Addr::new(192, 168, 1, 100);

        manager.add_to_blocklist(ip, 100); // Expires at time 100

        // Before expiration
        manager.cleanup_expired(50);
        assert!(manager.is_blocked(ip));

        // After expiration
        manager.cleanup_expired(150);
        assert!(!manager.is_blocked(ip));
    }

    /// Test permanent blocklist entry (expires_at = 0)
    #[test]
    fn test_permanent_block() {
        let mut manager = MockMapManager::new();
        let ip = Ipv4Addr::new(192, 168, 1, 100);

        manager.add_to_blocklist(ip, 0); // Permanent

        manager.cleanup_expired(u64::MAX);
        assert!(manager.is_blocked(ip)); // Still blocked
    }
}

// ============================================================================
// Allowlist Tests
// ============================================================================

#[cfg(test)]
mod allowlist_tests {
    use super::*;

    /// Test adding IP to allowlist
    #[test]
    fn test_add_to_allowlist() {
        let mut manager = MockMapManager::new();
        let ip = Ipv4Addr::new(10, 0, 0, 1);

        manager.add_to_allowlist(ip, 0);

        assert!(manager.is_allowed(ip));
    }

    /// Test removing IP from allowlist
    #[test]
    fn test_remove_from_allowlist() {
        let mut manager = MockMapManager::new();
        let ip = Ipv4Addr::new(10, 0, 0, 1);

        manager.add_to_allowlist(ip, 0);
        let removed = manager.remove_from_allowlist(ip);

        assert!(removed);
        assert!(!manager.is_allowed(ip));
    }

    /// Test allowlist expiration
    #[test]
    fn test_allowlist_expiration() {
        let mut manager = MockMapManager::new();
        let ip = Ipv4Addr::new(10, 0, 0, 1);

        manager.add_to_allowlist(ip, 100);

        manager.cleanup_expired(150);
        assert!(!manager.is_allowed(ip));
    }
}

// ============================================================================
// Rate Limit Tests
// ============================================================================

#[cfg(test)]
mod rate_limit_tests {
    use super::*;

    /// Test setting rate limit
    #[test]
    fn test_set_rate_limit() {
        let mut manager = MockMapManager::new();
        let ip = Ipv4Addr::new(192, 168, 1, 100);

        manager.set_rate_limit(ip, 100);

        assert!(manager.check_rate_limit(ip));
    }

    /// Test consuming tokens
    #[test]
    fn test_consume_tokens() {
        let mut manager = MockMapManager::new();
        let ip = Ipv4Addr::new(192, 168, 1, 100);

        manager.set_rate_limit(ip, 3);

        assert!(manager.consume_token(ip)); // 2 left
        assert!(manager.consume_token(ip)); // 1 left
        assert!(manager.consume_token(ip)); // 0 left
        assert!(!manager.consume_token(ip)); // Denied
    }

    /// Test no rate limit allows all
    #[test]
    fn test_no_rate_limit() {
        let manager = MockMapManager::new();
        let ip = Ipv4Addr::new(192, 168, 1, 100);

        // No rate limit set
        assert!(manager.check_rate_limit(ip));
    }

    /// Test rate limit at zero
    #[test]
    fn test_rate_limit_zero() {
        let mut manager = MockMapManager::new();
        let ip = Ipv4Addr::new(192, 168, 1, 100);

        manager.set_rate_limit(ip, 0);

        assert!(!manager.check_rate_limit(ip));
        assert!(!manager.consume_token(ip));
    }
}

// ============================================================================
// Stats Tests
// ============================================================================

#[cfg(test)]
mod stats_tests {
    use super::*;

    /// Test incrementing stats
    #[test]
    fn test_increment_stats() {
        let mut manager = MockMapManager::new();
        let ip = Ipv4Addr::new(192, 168, 1, 100);

        manager.increment_stats(ip, 10, 1000, 2, 200);

        let stats = manager.get_stats(ip).unwrap();
        assert_eq!(stats.packets_total, 10);
        assert_eq!(stats.bytes_total, 1000);
        assert_eq!(stats.packets_dropped, 2);
        assert_eq!(stats.bytes_dropped, 200);
    }

    /// Test accumulating stats
    #[test]
    fn test_accumulate_stats() {
        let mut manager = MockMapManager::new();
        let ip = Ipv4Addr::new(192, 168, 1, 100);

        manager.increment_stats(ip, 10, 1000, 0, 0);
        manager.increment_stats(ip, 5, 500, 1, 100);

        let stats = manager.get_stats(ip).unwrap();
        assert_eq!(stats.packets_total, 15);
        assert_eq!(stats.bytes_total, 1500);
        assert_eq!(stats.packets_dropped, 1);
    }

    /// Test get stats for unknown IP
    #[test]
    fn test_stats_unknown() {
        let manager = MockMapManager::new();
        let ip = Ipv4Addr::new(192, 168, 1, 100);

        assert!(manager.get_stats(ip).is_none());
    }

    /// Test clearing stats
    #[test]
    fn test_clear_stats() {
        let mut manager = MockMapManager::new();
        let ip = Ipv4Addr::new(192, 168, 1, 100);

        manager.increment_stats(ip, 10, 1000, 0, 0);
        manager.clear_stats();

        assert!(manager.get_stats(ip).is_none());
    }
}

// ============================================================================
// CIDR Tests
// ============================================================================

#[cfg(test)]
mod cidr_tests {
    use super::*;

    /// Test CIDR to IP range conversion
    #[test]
    fn test_cidr_to_range() {
        // 192.168.1.0/24 should include 192.168.1.0 - 192.168.1.255
        let network = Ipv4Addr::new(192, 168, 1, 0);
        let prefix_len = 24;

        let mask = !((1u32 << (32 - prefix_len)) - 1);
        let network_u32 = u32::from_be_bytes(network.octets());
        let broadcast_u32 = network_u32 | !mask;

        assert_eq!(network_u32 & mask, network_u32);
        assert_eq!(broadcast_u32, u32::from_be_bytes([192, 168, 1, 255]));
    }

    /// Test IP in CIDR range check
    #[test]
    fn test_ip_in_cidr() {
        let network = Ipv4Addr::new(192, 168, 1, 0);
        let prefix_len = 24u32;
        let mask = !((1u32 << (32 - prefix_len)) - 1);

        let test_ip_in = Ipv4Addr::new(192, 168, 1, 100);
        let test_ip_out = Ipv4Addr::new(192, 168, 2, 1);

        let network_u32 = u32::from_be_bytes(network.octets());
        let test_in_u32 = u32::from_be_bytes(test_ip_in.octets());
        let test_out_u32 = u32::from_be_bytes(test_ip_out.octets());

        assert_eq!(test_in_u32 & mask, network_u32);
        assert_ne!(test_out_u32 & mask, network_u32);
    }
}

// ============================================================================
// Batch Operation Tests
// ============================================================================

#[cfg(test)]
mod batch_tests {
    use super::*;

    /// Test batch block operation
    #[test]
    fn test_batch_block() {
        let mut manager = MockMapManager::new();

        let ips: Vec<Ipv4Addr> = (1..=100).map(|i| Ipv4Addr::new(10, 0, 0, i)).collect();

        for ip in &ips {
            manager.add_to_blocklist(*ip, 0);
        }

        assert_eq!(manager.get_blocklist().len(), 100);
    }

    /// Test batch unblock operation
    #[test]
    fn test_batch_unblock() {
        let mut manager = MockMapManager::new();

        let ips: Vec<Ipv4Addr> = (1..=100).map(|i| Ipv4Addr::new(10, 0, 0, i)).collect();

        for ip in &ips {
            manager.add_to_blocklist(*ip, 0);
        }

        for ip in &ips {
            manager.remove_from_blocklist(*ip);
        }

        assert!(manager.get_blocklist().is_empty());
    }
}
