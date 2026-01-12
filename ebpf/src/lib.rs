//! PistonProtection eBPF/XDP Library
//!
//! This library provides shared types and utilities for the XDP filter programs.
//! Each XDP program is compiled as a separate binary for loading into the kernel.
//!
//! # Available XDP Programs
//!
//! - `xdp_filter` - Main XDP DDoS filter with general-purpose protection
//! - `xdp_ratelimit` - Dedicated rate limiting with token bucket algorithm
//! - `xdp_minecraft` - Specialized Minecraft Java/Bedrock protocol filtering
//! - `xdp_http` - HTTP/1.1 and HTTP/2 protocol filtering
//! - `xdp_quic` - QUIC (HTTP/3) protocol filtering
//! - `xdp_udp` - Generic UDP filtering with amplification detection
//! - `xdp_tcp` - Enhanced TCP filtering with SYN cookies
//!
//! # Architecture
//!
//! Each XDP program operates independently and can be attached to different
//! network interfaces or chained together using XDP_PASS/XDP_TX/XDP_REDIRECT.
//!
//! The programs share common map structures where appropriate, allowing
//! userspace to manage blocklists and configuration centrally.

#![no_std]

// ============================================================================
// Common Types
// ============================================================================

/// Block reasons for IP blocking
#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BlockReason {
    /// Manual block by administrator
    Manual = 0,
    /// Rate limit exceeded
    RateLimit = 1,
    /// SYN flood detected
    SynFlood = 2,
    /// ACK flood detected
    AckFlood = 3,
    /// RST flood detected
    RstFlood = 4,
    /// UDP flood detected
    UdpFlood = 5,
    /// ICMP flood detected
    IcmpFlood = 6,
    /// DNS amplification attack
    DnsAmplification = 7,
    /// NTP amplification attack
    NtpAmplification = 8,
    /// SSDP amplification attack
    SsdpAmplification = 9,
    /// Memcached amplification attack
    MemcachedAmplification = 10,
    /// Invalid protocol packets
    InvalidProtocol = 11,
    /// Port scan detected
    PortScan = 12,
    /// HTTP slow attack
    HttpSlowAttack = 13,
    /// HTTP rate limit
    HttpRateLimit = 14,
    /// QUIC amplification attack
    QuicAmplification = 15,
    /// Invalid QUIC version
    InvalidQuicVersion = 16,
    /// Connection limit exceeded
    ConnectionLimit = 17,
    /// Invalid Minecraft packets
    InvalidMinecraft = 18,
    /// Minecraft bot attack
    MinecraftBot = 19,
    /// Generic DDoS detection
    GenericDdos = 20,
}

/// Protection levels
#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ProtectionLevel {
    /// Minimal filtering, only obvious attacks
    Low = 1,
    /// Balanced filtering (recommended)
    Medium = 2,
    /// Aggressive filtering, may have false positives
    High = 3,
    /// Maximum protection, strict validation
    Maximum = 4,
}

/// XDP program identifiers
#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum XdpProgram {
    /// Main filter program
    Filter = 0,
    /// Rate limiting program
    RateLimit = 1,
    /// Minecraft protocol filter
    Minecraft = 2,
    /// HTTP protocol filter
    Http = 3,
    /// QUIC protocol filter
    Quic = 4,
    /// Generic UDP filter
    Udp = 5,
    /// Enhanced TCP filter
    Tcp = 6,
}

// ============================================================================
// Shared Map Key Types
// ============================================================================

/// IPv4 address key
pub type Ipv4Key = u32;

/// IPv6 address key
pub type Ipv6Key = [u8; 16];

/// Connection 4-tuple key (hashed)
pub type ConnectionKey = u64;

/// Port key
pub type PortKey = u16;

// ============================================================================
// Common Configuration Structures
// ============================================================================

/// Base configuration shared by all programs
#[repr(C)]
#[derive(Clone, Copy)]
pub struct BaseConfig {
    /// Program enabled flag
    pub enabled: u32,
    /// Protection level (1-4)
    pub protection_level: u32,
    /// Rate limit window in nanoseconds
    pub rate_limit_window_ns: u64,
    /// Block duration in nanoseconds
    pub block_duration_ns: u64,
}

impl Default for BaseConfig {
    fn default() -> Self {
        Self {
            enabled: 1,
            protection_level: 2,
            rate_limit_window_ns: 1_000_000_000, // 1 second
            block_duration_ns: 60_000_000_000,   // 60 seconds
        }
    }
}

// ============================================================================
// Common Statistics Structures
// ============================================================================

/// Base statistics shared by all programs
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct BaseStats {
    /// Total packets processed
    pub total_packets: u64,
    /// Packets passed through
    pub passed_packets: u64,
    /// Packets dropped
    pub dropped_packets: u64,
    /// Total bytes processed
    pub total_bytes: u64,
}

// ============================================================================
// Protocol Constants
// ============================================================================

pub mod protocol {
    /// Ethernet protocol types
    pub mod eth {
        pub const P_IP: u16 = 0x0800;
        pub const P_IPV6: u16 = 0x86DD;
        pub const P_ARP: u16 = 0x0806;
        pub const P_8021Q: u16 = 0x8100;
    }

    /// IP protocol numbers
    pub mod ip {
        pub const PROTO_ICMP: u8 = 1;
        pub const PROTO_TCP: u8 = 6;
        pub const PROTO_UDP: u8 = 17;
        pub const PROTO_ICMPV6: u8 = 58;
    }

    /// Common ports
    pub mod ports {
        pub const HTTP: u16 = 80;
        pub const HTTPS: u16 = 443;
        pub const DNS: u16 = 53;
        pub const NTP: u16 = 123;
        pub const SSDP: u16 = 1900;
        pub const SNMP: u16 = 161;
        pub const MEMCACHED: u16 = 11211;
        pub const MINECRAFT_JAVA: u16 = 25565;
        pub const MINECRAFT_BEDROCK: u16 = 19132;
    }

    /// TCP flags
    pub mod tcp {
        pub const FIN: u16 = 0x0001;
        pub const SYN: u16 = 0x0002;
        pub const RST: u16 = 0x0004;
        pub const PSH: u16 = 0x0008;
        pub const ACK: u16 = 0x0010;
        pub const URG: u16 = 0x0020;
        pub const ECE: u16 = 0x0040;
        pub const CWR: u16 = 0x0080;
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Calculate a simple hash for connection tracking
#[inline(always)]
pub fn hash_connection(
    src_ip: u32,
    dst_ip: u32,
    src_port: u16,
    dst_port: u16,
) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325; // FNV-1a offset basis

    // Mix in source IP
    hash ^= src_ip as u64;
    hash = hash.wrapping_mul(0x100000001b3);

    // Mix in destination IP
    hash ^= dst_ip as u64;
    hash = hash.wrapping_mul(0x100000001b3);

    // Mix in ports
    hash ^= ((src_port as u64) << 16) | (dst_port as u64);
    hash = hash.wrapping_mul(0x100000001b3);

    hash
}

/// Calculate a symmetric hash (same for both directions)
#[inline(always)]
pub fn hash_connection_symmetric(
    ip1: u32,
    ip2: u32,
    port1: u16,
    port2: u16,
) -> u64 {
    let (src_ip, dst_ip, src_port, dst_port) = if ip1 < ip2 {
        (ip1, ip2, port1, port2)
    } else if ip1 > ip2 {
        (ip2, ip1, port2, port1)
    } else if port1 < port2 {
        (ip1, ip2, port1, port2)
    } else {
        (ip2, ip1, port2, port1)
    };

    hash_connection(src_ip, dst_ip, src_port, dst_port)
}

// ============================================================================
// Map Names (for userspace coordination)
// ============================================================================

pub mod map_names {
    // xdp_filter maps
    pub const BLOCKED_IPS_V4: &str = "BLOCKED_IPS_V4";
    pub const BLOCKED_IPS_V6: &str = "BLOCKED_IPS_V6";
    pub const RATE_LIMITS_V4: &str = "RATE_LIMITS_V4";
    pub const RATE_LIMITS_V6: &str = "RATE_LIMITS_V6";
    pub const FILTER_CONFIG: &str = "CONFIG";
    pub const FILTER_STATS: &str = "STATS";

    // xdp_ratelimit maps
    pub const TOKEN_BUCKETS_V4: &str = "TOKEN_BUCKETS_V4";
    pub const TOKEN_BUCKETS_V6: &str = "TOKEN_BUCKETS_V6";
    pub const SUBNET_BUCKETS: &str = "SUBNET_BUCKETS";
    pub const RATELIMIT_CONFIG: &str = "RATELIMIT_CONFIG";
    pub const RATELIMIT_STATS: &str = "RATELIMIT_STATS";

    // xdp_minecraft maps
    pub const MC_JAVA_CONNECTIONS: &str = "MC_JAVA_CONNECTIONS";
    pub const MC_BEDROCK_CONNECTIONS: &str = "MC_BEDROCK_CONNECTIONS";
    pub const MC_IP_COUNTS: &str = "MC_IP_COUNTS";
    pub const MC_STATUS_RATE: &str = "MC_STATUS_RATE";
    pub const MC_CONFIG: &str = "MC_CONFIG";

    // xdp_http maps
    pub const HTTP_CONNECTIONS: &str = "HTTP_CONNECTIONS";
    pub const HTTP_RATE_LIMITS: &str = "HTTP_RATE_LIMITS";
    pub const HTTP_RATE_LIMITS_V6: &str = "HTTP_RATE_LIMITS_V6";
    pub const BLOCKED_PATHS: &str = "BLOCKED_PATHS";
    pub const BLOCKED_USER_AGENTS: &str = "BLOCKED_USER_AGENTS";
    pub const HTTP_WHITELIST: &str = "HTTP_WHITELIST";
    pub const HTTP_CONFIG: &str = "HTTP_CONFIG";
    pub const HTTP_STATS: &str = "HTTP_STATS";

    // xdp_quic maps
    pub const QUIC_CONNECTIONS: &str = "QUIC_CONNECTIONS";
    pub const QUIC_RATE_LIMITS_V4: &str = "QUIC_RATE_LIMITS_V4";
    pub const QUIC_RATE_LIMITS_V6: &str = "QUIC_RATE_LIMITS_V6";
    pub const QUIC_VALID_CIDS: &str = "QUIC_VALID_CIDS";
    pub const QUIC_WHITELIST: &str = "QUIC_WHITELIST";
    pub const QUIC_CONFIG: &str = "QUIC_CONFIG";
    pub const QUIC_STATS: &str = "QUIC_STATS";

    // xdp_udp maps
    pub const UDP_IP_STATE_V4: &str = "UDP_IP_STATE_V4";
    pub const UDP_IP_STATE_V6: &str = "UDP_IP_STATE_V6";
    pub const UDP_PORT_STATE: &str = "UDP_PORT_STATE";
    pub const AMP_SOURCES: &str = "AMP_SOURCES";
    pub const BLOCKED_PORTS: &str = "BLOCKED_PORTS";
    pub const UDP_WHITELIST: &str = "UDP_WHITELIST";
    pub const PROTECTED_PORTS: &str = "PROTECTED_PORTS";
    pub const UDP_CONFIG: &str = "UDP_CONFIG";
    pub const UDP_STATS: &str = "UDP_STATS";

    // xdp_tcp maps
    pub const TCP_CONNECTIONS: &str = "TCP_CONNECTIONS";
    pub const TCP_IP_STATE_V4: &str = "TCP_IP_STATE_V4";
    pub const TCP_IP_STATE_V6: &str = "TCP_IP_STATE_V6";
    pub const SYN_COOKIES: &str = "SYN_COOKIES";
    pub const GLOBAL_SYN_STATE: &str = "GLOBAL_SYN_STATE";
    pub const TCP_PROTECTED_PORTS: &str = "TCP_PROTECTED_PORTS";
    pub const TCP_WHITELIST: &str = "TCP_WHITELIST";
    pub const TCP_CONFIG: &str = "TCP_CONFIG";
    pub const TCP_STATS: &str = "TCP_STATS";
}
