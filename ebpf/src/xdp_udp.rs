//! XDP Generic UDP Filter
//!
//! XDP program for generic UDP filtering with:
//! - Packet size validation
//! - Amplification attack detection (NTP, DNS, SSDP, Memcached, etc.)
//! - UDP flood mitigation
//! - Port scan detection
//! - Reflection attack prevention

#![no_std]
#![no_main]

use aya_ebpf::{
    bindings::xdp_action,
    macros::{map, xdp},
    maps::{HashMap, LruHashMap, PerCpuArray},
    programs::XdpContext,
};
use core::mem;

// ============================================================================
// Network Header Structures
// ============================================================================

#[repr(C)]
struct EthHdr {
    h_dest: [u8; 6],
    h_source: [u8; 6],
    h_proto: u16,
}

#[repr(C)]
struct Ipv4Hdr {
    version_ihl: u8,
    tos: u8,
    tot_len: u16,
    id: u16,
    frag_off: u16,
    ttl: u8,
    protocol: u8,
    check: u16,
    saddr: u32,
    daddr: u32,
}

#[repr(C)]
struct Ipv6Hdr {
    version_tc_flow: u32,
    payload_len: u16,
    nexthdr: u8,
    hop_limit: u8,
    saddr: [u8; 16],
    daddr: [u8; 16],
}

#[repr(C)]
struct UdpHdr {
    source: u16,
    dest: u16,
    len: u16,
    check: u16,
}

// ============================================================================
// UDP Filtering Structures
// ============================================================================

/// Per-IP UDP statistics and rate limiting
#[repr(C)]
pub struct UdpIpState {
    /// Total packets from this IP
    pub packets: u64,
    /// Total bytes from this IP
    pub bytes: u64,
    /// Window start timestamp
    pub window_start: u64,
    /// Packets in current window
    pub window_packets: u64,
    /// Last seen timestamp
    pub last_seen: u64,
    /// Unique destination ports seen (port scan detection)
    pub unique_ports: u32,
    /// Amplification response packets
    pub amp_responses: u32,
    /// Blocked until timestamp
    pub blocked_until: u64,
    /// Flags
    pub flags: u32,
    /// Bloom filter for tracking unique ports (64 bytes = 512 bits)
    /// Uses 3 hash functions for good collision resistance
    pub port_bloom_filter: [u64; 8],
}

/// Per-port statistics (for detecting targeted attacks)
#[repr(C)]
pub struct UdpPortState {
    /// Packets to this port
    pub packets: u64,
    /// Unique source IPs
    pub unique_sources: u32,
    /// Window start
    pub window_start: u64,
    /// Packets in window
    pub window_packets: u64,
}

/// UDP filter configuration
#[repr(C)]
#[derive(Copy, Clone)]
pub struct UdpConfig {
    /// Filter enabled
    pub enabled: u32,
    /// Minimum UDP packet size (excluding headers)
    pub min_packet_size: u16,
    /// Maximum UDP packet size
    pub max_packet_size: u16,
    /// Rate limit window (nanoseconds)
    pub rate_limit_window_ns: u64,
    /// Maximum packets per IP per window
    pub max_packets_per_window: u64,
    /// Maximum bytes per IP per window
    pub max_bytes_per_window: u64,
    /// Block duration (nanoseconds)
    pub block_duration_ns: u64,
    /// Protection level (1=basic, 2=moderate, 3=aggressive)
    pub protection_level: u32,
    /// Enable amplification detection
    pub amp_detection_enabled: u32,
    /// Enable port scan detection
    pub portscan_detection_enabled: u32,
    /// Port scan threshold (unique ports per window)
    pub portscan_threshold: u32,
}

/// UDP statistics
#[repr(C)]
pub struct UdpStats {
    pub total_packets: u64,
    pub passed_packets: u64,
    pub dropped_rate_limited: u64,
    pub dropped_invalid_size: u64,
    pub dropped_amplification: u64,
    pub dropped_port_scan: u64,
    pub dropped_blocked_ip: u64,
    pub dropped_blocked_port: u64,
    pub dropped_fragmented: u64,
    pub dns_packets: u64,
    pub ntp_packets: u64,
    pub ssdp_packets: u64,
    pub memcached_packets: u64,
}

/// Amplification source tracking
#[repr(C)]
pub struct AmpSourceEntry {
    /// First seen timestamp
    pub first_seen: u64,
    /// Total packets
    pub packets: u64,
    /// Total response bytes
    pub response_bytes: u64,
    /// Blocked until
    pub blocked_until: u64,
}

// ============================================================================
// Constants
// ============================================================================

const ETH_P_IP: u16 = 0x0800;
const ETH_P_IPV6: u16 = 0x86DD;
const IPPROTO_UDP: u8 = 17;
const IPPROTO_FRAGMENT: u8 = 44; // IPv6 Fragment header next protocol

// IPv4 fragmentation flags (in frag_off field)
// frag_off is 16 bits: [3 bits flags][13 bits fragment offset]
// Flags: bit 15 = Reserved, bit 14 = DF (Don't Fragment), bit 13 = MF (More Fragments)
const IP_MF: u16 = 0x2000; // More Fragments flag
const IP_OFFSET_MASK: u16 = 0x1FFF; // Fragment offset mask (13 bits)

// Amplification attack source ports
const PORT_DNS: u16 = 53;
const PORT_NTP: u16 = 123;
const PORT_SSDP: u16 = 1900;
const PORT_SNMP: u16 = 161;
const PORT_MEMCACHED: u16 = 11211;
const PORT_CHARGEN: u16 = 19;
const PORT_QOTD: u16 = 17;
const PORT_LDAP: u16 = 389;
const PORT_MSSQL: u16 = 1434;
const PORT_RIP: u16 = 520;
const PORT_PORTMAP: u16 = 111;
const PORT_NETBIOS: u16 = 137;
const PORT_CLDAP: u16 = 636;
const PORT_TFTP: u16 = 69;
const PORT_OPENVPN: u16 = 1194;
const PORT_STEAM: u16 = 27015;

// DNS-specific constants
const DNS_FLAG_RESPONSE: u16 = 0x8000;
const DNS_FLAG_RECURSION_AVAILABLE: u16 = 0x0080;

// NTP-specific constants
const NTP_MODE_MASK: u8 = 0x07;
const NTP_MODE_SERVER: u8 = 4;
const NTP_MODE_BROADCAST: u8 = 5;

// State flags
const FLAG_AMP_DETECTED: u32 = 0x0001;
const FLAG_PORTSCAN_DETECTED: u32 = 0x0002;
const FLAG_FLOOD_DETECTED: u32 = 0x0004;

// Default configuration
const DEFAULT_MIN_PACKET_SIZE: u16 = 0;
const DEFAULT_MAX_PACKET_SIZE: u16 = 65535;
const DEFAULT_RATE_LIMIT_WINDOW_NS: u64 = 1_000_000_000; // 1 second
const DEFAULT_MAX_PACKETS_PER_WINDOW: u64 = 1000;
const DEFAULT_MAX_BYTES_PER_WINDOW: u64 = 1_000_000; // 1MB
const DEFAULT_BLOCK_DURATION_NS: u64 = 60_000_000_000; // 60 seconds
const DEFAULT_PORTSCAN_THRESHOLD: u32 = 50;

// ============================================================================
// eBPF Maps
// ============================================================================

/// Per-IP UDP state (IPv4)
#[map]
static UDP_IP_STATE_V4: LruHashMap<u32, UdpIpState> = LruHashMap::with_max_entries(1_000_000, 0);

/// Per-IP UDP state (IPv6)
#[map]
static UDP_IP_STATE_V6: LruHashMap<[u8; 16], UdpIpState> = LruHashMap::with_max_entries(500_000, 0);

/// Per-port state (destination ports)
#[map]
static UDP_PORT_STATE: LruHashMap<u16, UdpPortState> = LruHashMap::with_max_entries(65536, 0);

/// Amplification source tracking (by source IP + source port)
#[map]
static AMP_SOURCES: LruHashMap<u64, AmpSourceEntry> = LruHashMap::with_max_entries(100_000, 0);

/// Blocked destination ports
#[map]
static BLOCKED_PORTS: HashMap<u16, u32> = HashMap::with_max_entries(1000, 0);

/// Whitelisted source IPs
#[map]
static UDP_WHITELIST: HashMap<u32, u32> = HashMap::with_max_entries(10_000, 0);

/// Protected destination ports (stricter filtering)
#[map]
static PROTECTED_PORTS: HashMap<u16, u32> = HashMap::with_max_entries(1000, 0);

/// Configuration
#[map]
static UDP_CONFIG: PerCpuArray<UdpConfig> = PerCpuArray::with_max_entries(1, 0);

/// Statistics
#[map]
static UDP_STATS: PerCpuArray<UdpStats> = PerCpuArray::with_max_entries(1, 0);

// ============================================================================
// Main XDP Entry Point
// ============================================================================

#[xdp]
pub fn xdp_udp(ctx: XdpContext) -> u32 {
    match try_xdp_udp(ctx) {
        Ok(ret) => ret,
        Err(_) => xdp_action::XDP_PASS,
    }
}

#[inline(always)]
fn try_xdp_udp(ctx: XdpContext) -> Result<u32, ()> {
    let config = get_config();
    if config.enabled == 0 {
        return Ok(xdp_action::XDP_PASS);
    }

    let data = ctx.data();
    let data_end = ctx.data_end();

    // Parse Ethernet header
    if data + mem::size_of::<EthHdr>() > data_end {
        return Ok(xdp_action::XDP_PASS);
    }

    let eth = unsafe { &*(data as *const EthHdr) };
    let eth_proto = u16::from_be(eth.h_proto);

    match eth_proto {
        ETH_P_IP => process_ipv4(&ctx, data + mem::size_of::<EthHdr>(), data_end, &config),
        ETH_P_IPV6 => process_ipv6(&ctx, data + mem::size_of::<EthHdr>(), data_end, &config),
        _ => Ok(xdp_action::XDP_PASS),
    }
}

// ============================================================================
// IPv4 Processing
// ============================================================================

#[inline(always)]
fn process_ipv4(
    ctx: &XdpContext,
    data: usize,
    data_end: usize,
    config: &UdpConfig,
) -> Result<u32, ()> {
    if data + mem::size_of::<Ipv4Hdr>() > data_end {
        return Ok(xdp_action::XDP_PASS);
    }

    let ip = unsafe { &*(data as *const Ipv4Hdr) };

    // Only process UDP
    if ip.protocol != IPPROTO_UDP {
        return Ok(xdp_action::XDP_PASS);
    }

    // ========================================================================
    // FRAGMENTATION HANDLING
    // ========================================================================
    // IP fragmentation can be used to bypass UDP filtering:
    // 1. First fragment contains UDP header but may have truncated payload
    // 2. Subsequent fragments have no UDP header, just raw payload
    // 3. Amplification attacks can use fragments to bypass size checks
    //
    // Strategy:
    // - Drop non-first fragments (no UDP header to inspect)
    // - For first fragments with MF=1, apply stricter validation
    // - Block known amplification port responses that are fragmented
    // ========================================================================
    let frag_off = u16::from_be(ip.frag_off);
    let is_fragmented = (frag_off & IP_MF) != 0 || (frag_off & IP_OFFSET_MASK) != 0;
    let is_first_fragment = (frag_off & IP_OFFSET_MASK) == 0;

    if is_fragmented {
        if !is_first_fragment {
            // Non-first fragment - has no UDP header, can't inspect
            // Drop at protection level >= 2 (moderate/aggressive)
            if config.protection_level >= 2 {
                update_stats_fragmented();
                return Ok(xdp_action::XDP_DROP);
            }
            // At protection level 1, pass through (may be legitimate)
            return Ok(xdp_action::XDP_PASS);
        }

        // First fragment with more fragments flag set
        // This is suspicious for UDP - legitimate UDP rarely fragments
        // At aggressive protection, drop all fragmented UDP
        if config.protection_level >= 3 {
            update_stats_fragmented();
            return Ok(xdp_action::XDP_DROP);
        }
        // Otherwise continue to process, but with heightened suspicion
        // The UDP processing will still validate what we can see
    }

    let src_ip = u32::from_be(ip.saddr);

    // Check whitelist
    if unsafe { UDP_WHITELIST.get(&src_ip) }.is_some() {
        return Ok(xdp_action::XDP_PASS);
    }

    // Check if IP is blocked
    if is_ip_blocked_v4(src_ip) {
        update_stats_blocked();
        return Ok(xdp_action::XDP_DROP);
    }

    let ihl = (ip.version_ihl & 0x0f) as usize * 4;
    let udp_data = data + ihl;

    // For fragmented first fragments, pass is_fragmented flag for stricter checks
    process_udp(ctx, udp_data, data_end, src_ip, config, is_fragmented)
}

// ============================================================================
// IPv6 Processing
// ============================================================================

/// IPv6 Fragment Header structure
#[repr(C)]
struct Ipv6FragHdr {
    nexthdr: u8,
    reserved: u8,
    frag_off_m: u16, // Fragment offset (13 bits) + Reserved (2 bits) + M flag (1 bit)
    identification: u32,
}

// IPv6 fragmentation constants
const IPV6_FRAG_OFFSET_MASK: u16 = 0xFFF8; // Upper 13 bits (shifted left by 3)
const IPV6_FRAG_M_FLAG: u16 = 0x0001; // More fragments flag (lowest bit)

#[inline(always)]
fn process_ipv6(
    ctx: &XdpContext,
    data: usize,
    data_end: usize,
    config: &UdpConfig,
) -> Result<u32, ()> {
    if data + mem::size_of::<Ipv6Hdr>() > data_end {
        return Ok(xdp_action::XDP_PASS);
    }

    let ip6 = unsafe { &*(data as *const Ipv6Hdr) };
    let mut next_header = ip6.nexthdr;
    let mut header_offset = data + mem::size_of::<Ipv6Hdr>();
    let mut is_fragmented = false;
    let mut is_first_fragment = true;

    // ========================================================================
    // IPv6 EXTENSION HEADER PARSING
    // ========================================================================
    // IPv6 can have extension headers between the main header and UDP.
    // We need to check for the Fragment extension header (protocol 44).
    // For simplicity, we only handle: Hop-by-Hop (0), Routing (43), Fragment (44)
    // Other extension headers are rare for UDP traffic.
    // ========================================================================

    // Bounded loop for eBPF verifier - max 3 extension headers is reasonable
    for _ in 0..3 {
        match next_header {
            IPPROTO_UDP => {
                // Found UDP, done parsing extension headers
                break;
            }
            IPPROTO_FRAGMENT => {
                // Fragment extension header
                if header_offset + mem::size_of::<Ipv6FragHdr>() > data_end {
                    return Ok(xdp_action::XDP_PASS);
                }

                let frag_hdr = unsafe { &*(header_offset as *const Ipv6FragHdr) };
                let frag_off_m = u16::from_be(frag_hdr.frag_off_m);

                // Check fragment offset (bits 3-15) and M flag (bit 0)
                let frag_offset = frag_off_m & IPV6_FRAG_OFFSET_MASK;
                let more_fragments = (frag_off_m & IPV6_FRAG_M_FLAG) != 0;

                is_fragmented = more_fragments || frag_offset != 0;
                is_first_fragment = frag_offset == 0;

                next_header = frag_hdr.nexthdr;
                header_offset += mem::size_of::<Ipv6FragHdr>();
            }
            0 | 43 | 60 => {
                // Hop-by-Hop (0), Routing (43), Destination Options (60)
                // These have a common format: next_header, length, data
                if header_offset + 2 > data_end {
                    return Ok(xdp_action::XDP_PASS);
                }

                let ext_next = unsafe { *(header_offset as *const u8) };
                let ext_len = unsafe { *((header_offset + 1) as *const u8) };
                // Length is in 8-byte units, not including first 8 bytes
                let total_len = ((ext_len as usize) + 1) * 8;

                next_header = ext_next;
                header_offset += total_len;
            }
            _ => {
                // Unknown or unsupported extension header
                // If it's not UDP, we can't process it
                return Ok(xdp_action::XDP_PASS);
            }
        }
    }

    // After parsing, check if we found UDP
    if next_header != IPPROTO_UDP {
        return Ok(xdp_action::XDP_PASS);
    }

    // ========================================================================
    // IPv6 FRAGMENTATION HANDLING
    // ========================================================================
    // Same strategy as IPv4: drop non-first fragments, be suspicious of first fragments
    // ========================================================================
    if is_fragmented {
        if !is_first_fragment {
            // Non-first fragment - has no UDP header, can't inspect
            if config.protection_level >= 2 {
                update_stats_fragmented();
                return Ok(xdp_action::XDP_DROP);
            }
            return Ok(xdp_action::XDP_PASS);
        }

        // First fragment with more fragments
        if config.protection_level >= 3 {
            update_stats_fragmented();
            return Ok(xdp_action::XDP_DROP);
        }
    }

    let src_ip = ip6.saddr;

    // Check if IP is blocked (using full IPv6 address)
    if is_ip_blocked_v6(&src_ip) {
        update_stats_blocked();
        return Ok(xdp_action::XDP_DROP);
    }

    // Use the full IPv6 address for proper rate limiting
    process_udp_v6(ctx, header_offset, data_end, &src_ip, config, is_fragmented)
}

// ============================================================================
// UDP Processing
// ============================================================================

#[inline(always)]
fn process_udp(
    ctx: &XdpContext,
    data: usize,
    data_end: usize,
    src_ip: u32,
    config: &UdpConfig,
    is_fragmented: bool,
) -> Result<u32, ()> {
    if data + mem::size_of::<UdpHdr>() > data_end {
        return Ok(xdp_action::XDP_PASS);
    }

    let udp = unsafe { &*(data as *const UdpHdr) };
    let src_port = u16::from_be(udp.source);
    let dst_port = u16::from_be(udp.dest);
    let udp_len = u16::from_be(udp.len);

    // Update stats
    update_stats_total();

    // Check for blocked destination port
    if unsafe { BLOCKED_PORTS.get(&dst_port) }.is_some() {
        update_stats_blocked_port();
        return Ok(xdp_action::XDP_DROP);
    }

    // ========================================================================
    // FRAGMENTED AMPLIFICATION CHECK
    // ========================================================================
    // If this is a fragmented packet from a known amplification port,
    // drop it immediately - legitimate services don't typically send
    // fragmented UDP responses.
    // ========================================================================
    if is_fragmented {
        let is_amp_source = matches!(
            src_port,
            PORT_DNS
                | PORT_NTP
                | PORT_SSDP
                | PORT_SNMP
                | PORT_MEMCACHED
                | PORT_CHARGEN
                | PORT_QOTD
                | PORT_LDAP
                | PORT_MSSQL
                | PORT_RIP
                | PORT_PORTMAP
                | PORT_NETBIOS
                | PORT_CLDAP
                | PORT_TFTP
        );

        if is_amp_source && config.protection_level >= 2 {
            // Fragmented response from amplification port - almost certainly an attack
            update_stats_amplification();
            update_stats_fragmented();
            return Ok(xdp_action::XDP_DROP);
        }
    }

    // Validate packet size
    let payload_len = udp_len.saturating_sub(8); // UDP header is 8 bytes

    let min_size = if config.min_packet_size != 0 {
        config.min_packet_size
    } else {
        DEFAULT_MIN_PACKET_SIZE
    };

    let max_size = if config.max_packet_size != 0 {
        config.max_packet_size
    } else {
        DEFAULT_MAX_PACKET_SIZE
    };

    if payload_len < min_size || payload_len > max_size {
        update_stats_invalid_size();
        return Ok(xdp_action::XDP_DROP);
    }

    // Check rate limit
    let now = unsafe { aya_ebpf::helpers::bpf_ktime_get_ns() };

    if !check_rate_limit_v4(src_ip, udp_len as u64, now, config) {
        update_stats_rate_limited();
        return Ok(xdp_action::XDP_DROP);
    }

    // Amplification attack detection
    if config.amp_detection_enabled != 0 {
        if let Some(action) = check_amplification_attack(
            ctx,
            data,
            data_end,
            src_ip,
            src_port,
            dst_port,
            payload_len,
            config,
            is_fragmented,
        ) {
            return Ok(action);
        }
    }

    // Port scan detection
    if config.portscan_detection_enabled != 0 {
        if is_port_scan(src_ip, dst_port, now, config) {
            update_stats_port_scan();
            if config.protection_level >= 2 {
                block_ip_v4(src_ip, config.block_duration_ns);
                return Ok(xdp_action::XDP_DROP);
            }
        }
    }

    // Protocol-specific tracking (for stats)
    track_protocol_stats(src_port, dst_port);

    update_stats_passed();
    Ok(xdp_action::XDP_PASS)
}

// ============================================================================
// IPv6 UDP Processing
// ============================================================================

/// Hash a full IPv6 address to a u32 for amplification tracking
/// Uses FNV-1a hash for good distribution
#[inline(always)]
fn hash_ipv6_to_u32(addr: &[u8; 16]) -> u32 {
    // FNV-1a hash constants for 32-bit
    const FNV_OFFSET: u32 = 0x811c9dc5;
    const FNV_PRIME: u32 = 0x01000193;

    let mut hash = FNV_OFFSET;

    // Unrolled loop for eBPF verifier
    hash ^= addr[0] as u32;
    hash = hash.wrapping_mul(FNV_PRIME);
    hash ^= addr[1] as u32;
    hash = hash.wrapping_mul(FNV_PRIME);
    hash ^= addr[2] as u32;
    hash = hash.wrapping_mul(FNV_PRIME);
    hash ^= addr[3] as u32;
    hash = hash.wrapping_mul(FNV_PRIME);
    hash ^= addr[4] as u32;
    hash = hash.wrapping_mul(FNV_PRIME);
    hash ^= addr[5] as u32;
    hash = hash.wrapping_mul(FNV_PRIME);
    hash ^= addr[6] as u32;
    hash = hash.wrapping_mul(FNV_PRIME);
    hash ^= addr[7] as u32;
    hash = hash.wrapping_mul(FNV_PRIME);
    hash ^= addr[8] as u32;
    hash = hash.wrapping_mul(FNV_PRIME);
    hash ^= addr[9] as u32;
    hash = hash.wrapping_mul(FNV_PRIME);
    hash ^= addr[10] as u32;
    hash = hash.wrapping_mul(FNV_PRIME);
    hash ^= addr[11] as u32;
    hash = hash.wrapping_mul(FNV_PRIME);
    hash ^= addr[12] as u32;
    hash = hash.wrapping_mul(FNV_PRIME);
    hash ^= addr[13] as u32;
    hash = hash.wrapping_mul(FNV_PRIME);
    hash ^= addr[14] as u32;
    hash = hash.wrapping_mul(FNV_PRIME);
    hash ^= addr[15] as u32;
    hash = hash.wrapping_mul(FNV_PRIME);

    hash
}

#[inline(always)]
fn process_udp_v6(
    ctx: &XdpContext,
    data: usize,
    data_end: usize,
    src_ip: &[u8; 16],
    config: &UdpConfig,
    is_fragmented: bool,
) -> Result<u32, ()> {
    if data + mem::size_of::<UdpHdr>() > data_end {
        return Ok(xdp_action::XDP_PASS);
    }

    let udp = unsafe { &*(data as *const UdpHdr) };
    let src_port = u16::from_be(udp.source);
    let dst_port = u16::from_be(udp.dest);
    let udp_len = u16::from_be(udp.len);

    // Update stats
    update_stats_total();

    // Check for blocked destination port
    if unsafe { BLOCKED_PORTS.get(&dst_port) }.is_some() {
        update_stats_blocked_port();
        return Ok(xdp_action::XDP_DROP);
    }

    // Fragmented amplification check (same as IPv4)
    if is_fragmented {
        let is_amp_source = matches!(
            src_port,
            PORT_DNS
                | PORT_NTP
                | PORT_SSDP
                | PORT_SNMP
                | PORT_MEMCACHED
                | PORT_CHARGEN
                | PORT_QOTD
                | PORT_LDAP
                | PORT_MSSQL
                | PORT_RIP
                | PORT_PORTMAP
                | PORT_NETBIOS
                | PORT_CLDAP
                | PORT_TFTP
        );

        if is_amp_source && config.protection_level >= 2 {
            update_stats_amplification();
            update_stats_fragmented();
            return Ok(xdp_action::XDP_DROP);
        }
    }

    // Validate packet size
    let payload_len = udp_len.saturating_sub(8);

    let min_size = if config.min_packet_size != 0 {
        config.min_packet_size
    } else {
        DEFAULT_MIN_PACKET_SIZE
    };

    let max_size = if config.max_packet_size != 0 {
        config.max_packet_size
    } else {
        DEFAULT_MAX_PACKET_SIZE
    };

    if payload_len < min_size || payload_len > max_size {
        update_stats_invalid_size();
        return Ok(xdp_action::XDP_DROP);
    }

    // Check rate limit using full IPv6 address
    let now = unsafe { aya_ebpf::helpers::bpf_ktime_get_ns() };

    if !check_rate_limit_v6(src_ip, udp_len as u64, now, config) {
        update_stats_rate_limited();
        return Ok(xdp_action::XDP_DROP);
    }

    // Amplification attack detection
    // Use hashed IPv6 address for amplification tracking (amp key uses u32)
    if config.amp_detection_enabled != 0 {
        let ip_hash = hash_ipv6_to_u32(src_ip);
        if let Some(action) = check_amplification_attack(
            ctx,
            data,
            data_end,
            ip_hash,
            src_port,
            dst_port,
            payload_len,
            config,
            is_fragmented,
        ) {
            return Ok(action);
        }
    }

    // Port scan detection using full IPv6 address
    if config.portscan_detection_enabled != 0 {
        if is_port_scan_v6(src_ip, dst_port, now, config) {
            update_stats_port_scan();
            if config.protection_level >= 2 {
                block_ip_v6(src_ip, config.block_duration_ns);
                return Ok(xdp_action::XDP_DROP);
            }
        }
    }

    // Protocol-specific tracking
    track_protocol_stats(src_port, dst_port);

    update_stats_passed();
    Ok(xdp_action::XDP_PASS)
}

// ============================================================================
// Amplification Attack Detection
// ============================================================================

#[inline(always)]
fn check_amplification_attack(
    _ctx: &XdpContext,
    data: usize,
    data_end: usize,
    src_ip: u32,
    src_port: u16,
    _dst_port: u16,
    payload_len: u16,
    config: &UdpConfig,
    is_fragmented: bool,
) -> Option<u32> {
    // Check if source port is a known amplification vector
    let is_amp_source = matches!(
        src_port,
        PORT_DNS
            | PORT_NTP
            | PORT_SSDP
            | PORT_SNMP
            | PORT_MEMCACHED
            | PORT_CHARGEN
            | PORT_QOTD
            | PORT_LDAP
            | PORT_MSSQL
            | PORT_RIP
            | PORT_PORTMAP
            | PORT_NETBIOS
            | PORT_CLDAP
            | PORT_TFTP
    );

    if !is_amp_source {
        return None;
    }

    let payload_start = data + mem::size_of::<UdpHdr>();

    // Protocol-specific validation
    match src_port {
        PORT_DNS => {
            // DNS amplification detection with proper protocol validation
            // DNS header structure (12 bytes minimum):
            // - Transaction ID: 2 bytes
            // - Flags: 2 bytes (QR, Opcode, AA, TC, RD, RA, Z, RCODE)
            // - QDCOUNT: 2 bytes (number of questions)
            // - ANCOUNT: 2 bytes (number of answers)
            // - NSCOUNT: 2 bytes (number of authority records)
            // - ARCOUNT: 2 bytes (number of additional records)

            if payload_start + 12 <= data_end {
                let flags = unsafe { u16::from_be(*((payload_start + 2) as *const u16)) };
                let qdcount = unsafe { u16::from_be(*((payload_start + 4) as *const u16)) };
                let ancount = unsafe { u16::from_be(*((payload_start + 6) as *const u16)) };

                // Check if QR bit is set (response)
                let is_response = (flags & DNS_FLAG_RESPONSE) != 0;

                // Extract opcode (bits 11-14 of flags)
                // Valid opcodes: 0=Query, 1=IQuery, 2=Status, 4=Notify, 5=Update
                let opcode = (flags >> 11) & 0x0F;
                let valid_opcode = opcode <= 5;

                // Check for amplification indicators:
                // 1. Response with many more answers than questions
                // 2. Large payload size
                // 3. ANY query responses (opcode 0 with many answers)
                let amp_ratio_suspicious = ancount > 10 && qdcount <= 2;
                let is_large = payload_len > 512;

                if is_response && valid_opcode {
                    // Amplification heuristics:
                    // - High answer/question ratio indicates amplification
                    // - Large responses (>512 bytes) are suspicious
                    // - ANY queries can return massive responses

                    let is_amplification = amp_ratio_suspicious || (is_large && ancount > qdcount * 5);

                    if is_amplification || (is_large && payload_len > 1024) {
                        update_stats_amplification();

                        let amp_key = ((src_ip as u64) << 16) | (src_port as u64);
                        track_amp_source(amp_key, payload_len as u64, config);

                        // Drop based on protection level and severity
                        if config.protection_level >= 2 {
                            // At moderate protection: drop highly suspicious responses
                            if amp_ratio_suspicious || payload_len > 1024 {
                                return Some(xdp_action::XDP_DROP);
                            }
                        }
                        if config.protection_level >= 3 && is_large {
                            // At aggressive protection: drop any large DNS response
                            return Some(xdp_action::XDP_DROP);
                        }
                    }
                }
            }
        }

        PORT_NTP => {
            // NTP amplification detection with mode 7 and version validation
            // NTP first byte structure:
            // - Bits 0-2: Mode (0-7)
            // - Bits 3-5: Version (1-4 are valid)
            // - Bits 6-7: Leap Indicator
            //
            // Mode 7 (private/monlist) is the main amplification vector
            // - "monlist" command returns list of last 600 clients
            // - Can amplify 200x or more
            //
            // Mode 6 (control) can also be abused but less common

            if payload_start + 1 <= data_end {
                let first_byte = unsafe { *(payload_start as *const u8) };
                let mode = first_byte & NTP_MODE_MASK;
                let version = (first_byte >> 3) & 0x07; // Bits 3-5

                // Validate NTP version (1-4 are valid, 0 and 5+ are invalid/suspicious)
                let valid_version = version >= 1 && version <= 4;

                // Mode 7 (private) - ALWAYS suspicious, this is the monlist attack vector
                // Drop immediately regardless of payload size
                if mode == 7 {
                    update_stats_amplification();
                    track_amp_source(
                        ((src_ip as u64) << 16) | (src_port as u64),
                        payload_len as u64,
                        config,
                    );

                    // Mode 7 traffic from external sources is almost always malicious
                    if config.protection_level >= 1 {
                        return Some(xdp_action::XDP_DROP);
                    }
                }

                // Mode 6 (control) - also suspicious, can leak info
                if mode == 6 && payload_len > 12 {
                    update_stats_amplification();
                    track_amp_source(
                        ((src_ip as u64) << 16) | (src_port as u64),
                        payload_len as u64,
                        config,
                    );

                    if config.protection_level >= 2 {
                        return Some(xdp_action::XDP_DROP);
                    }
                }

                // Check for server response or broadcast (standard NTP)
                if (mode == NTP_MODE_SERVER || mode == NTP_MODE_BROADCAST) && valid_version {
                    // Standard NTP response is 48 bytes
                    // Larger responses indicate potential abuse
                    if payload_len > 48 {
                        update_stats_amplification();
                        track_amp_source(
                            ((src_ip as u64) << 16) | (src_port as u64),
                            payload_len as u64,
                            config,
                        );

                        if config.protection_level >= 2 && payload_len > 200 {
                            return Some(xdp_action::XDP_DROP);
                        }
                    }
                }

                // Invalid version with any response mode is suspicious
                if !valid_version && (mode == NTP_MODE_SERVER || mode == NTP_MODE_BROADCAST || mode == 6 || mode == 7) {
                    update_stats_amplification();
                    if config.protection_level >= 2 {
                        return Some(xdp_action::XDP_DROP);
                    }
                }
            }
        }

        PORT_SSDP => {
            // SSDP amplification (typically large M-SEARCH responses)
            if payload_len > 200 {
                update_stats_amplification();
                track_amp_source(
                    ((src_ip as u64) << 16) | (src_port as u64),
                    payload_len as u64,
                    config,
                );

                if config.protection_level >= 2 {
                    return Some(xdp_action::XDP_DROP);
                }
            }
        }

        PORT_MEMCACHED => {
            // Memcached amplification detection
            // Only block RESPONSES (src_port == 11211), not requests (dst_port == 11211)
            // This check is implicitly satisfied since we're in the src_port match branch
            //
            // Check for memcached binary protocol magic bytes:
            // 0x80 = request magic (shouldn't come from server)
            // 0x81 = response magic (amplification indicator)
            // Text protocol responses start with "VALUE", "END", "STAT", etc.

            if payload_start + 1 <= data_end {
                let magic_byte = unsafe { *(payload_start as *const u8) };

                // Binary protocol response magic (0x81) or request magic echoed back (0x80)
                // Both indicate potential amplification
                let is_binary_protocol = magic_byte == 0x80 || magic_byte == 0x81;

                // Also check for large payloads which are characteristic of amplification
                // Memcached can return massive responses (up to 1MB+ per key)
                let is_large_response = payload_len > 100;

                if is_binary_protocol || is_large_response {
                    update_stats_amplification();
                    track_amp_source(
                        ((src_ip as u64) << 16) | (src_port as u64),
                        payload_len as u64,
                        config,
                    );

                    // Drop binary protocol responses or large text responses
                    if config.protection_level >= 1 {
                        if is_binary_protocol || payload_len > 500 {
                            return Some(xdp_action::XDP_DROP);
                        }
                    }
                }
            }
        }

        PORT_CHARGEN | PORT_QOTD => {
            // These should almost never be legitimate traffic
            update_stats_amplification();
            if config.protection_level >= 1 {
                return Some(xdp_action::XDP_DROP);
            }
        }

        PORT_SNMP => {
            // SNMP amplification
            if payload_len > 200 {
                update_stats_amplification();
                track_amp_source(
                    ((src_ip as u64) << 16) | (src_port as u64),
                    payload_len as u64,
                    config,
                );

                if config.protection_level >= 2 {
                    return Some(xdp_action::XDP_DROP);
                }
            }
        }

        PORT_LDAP | PORT_CLDAP => {
            // LDAP/CLDAP amplification
            if payload_len > 100 {
                update_stats_amplification();
                track_amp_source(
                    ((src_ip as u64) << 16) | (src_port as u64),
                    payload_len as u64,
                    config,
                );

                if config.protection_level >= 2 {
                    return Some(xdp_action::XDP_DROP);
                }
            }
        }

        _ => {
            // Generic large response from known amp port
            if payload_len > 500 {
                update_stats_amplification();
                track_amp_source(
                    ((src_ip as u64) << 16) | (src_port as u64),
                    payload_len as u64,
                    config,
                );
            }
        }
    }

    None
}

#[inline(always)]
fn track_amp_source(amp_key: u64, bytes: u64, config: &UdpConfig) {
    let now = unsafe { aya_ebpf::helpers::bpf_ktime_get_ns() };

    if let Some(entry) = unsafe { AMP_SOURCES.get_ptr_mut(&amp_key) } {
        let entry = unsafe { &mut *entry };
        entry.packets += 1;
        entry.response_bytes += bytes;

        // Auto-block if too many amplification packets
        if entry.packets > 100 || entry.response_bytes > 1_000_000 {
            entry.blocked_until = now + config.block_duration_ns;
        }
    } else {
        let entry = AmpSourceEntry {
            first_seen: now,
            packets: 1,
            response_bytes: bytes,
            blocked_until: 0,
        };
        let _ = AMP_SOURCES.insert(&amp_key, &entry, 0);
    }
}

// ============================================================================
// Port Scan Detection with Bloom Filter
// ============================================================================

/// Compute bloom filter hash indices for a port number
/// Uses 3 independent hash functions for good collision resistance
/// Returns (hash1, hash2, hash3) as bit indices into the 512-bit filter
#[inline(always)]
fn bloom_hash_port(port: u16) -> (usize, usize, usize) {
    let port32 = port as u32;

    // Hash function 1: Simple multiplication hash
    let h1 = (port32.wrapping_mul(0x9E3779B9)) & 0x1FF; // 9 bits for 512 positions

    // Hash function 2: Different multiplier
    let h2 = (port32.wrapping_mul(0x85EBCA6B).wrapping_add(0x3F)) & 0x1FF;

    // Hash function 3: XOR-based hash
    let h3 = ((port32 ^ (port32 >> 5)).wrapping_mul(0xC2B2AE35)) & 0x1FF;

    (h1 as usize, h2 as usize, h3 as usize)
}

/// Check if a port is in the bloom filter and add it if not
/// Returns true if the port was already present (likely), false if newly added
#[inline(always)]
fn bloom_check_and_add(filter: &mut [u64; 8], port: u16) -> bool {
    let (h1, h2, h3) = bloom_hash_port(port);

    // Calculate array index and bit position for each hash
    let idx1 = h1 >> 6; // Divide by 64 to get u64 index
    let bit1 = h1 & 0x3F; // Mod 64 to get bit position

    let idx2 = h2 >> 6;
    let bit2 = h2 & 0x3F;

    let idx3 = h3 >> 6;
    let bit3 = h3 & 0x3F;

    // Bounds check for eBPF verifier - indices are always < 8
    if idx1 >= 8 || idx2 >= 8 || idx3 >= 8 {
        return false;
    }

    // Check if all bits are already set (port likely already seen)
    let already_present = (filter[idx1] & (1u64 << bit1)) != 0
        && (filter[idx2] & (1u64 << bit2)) != 0
        && (filter[idx3] & (1u64 << bit3)) != 0;

    // Set all bits
    filter[idx1] |= 1u64 << bit1;
    filter[idx2] |= 1u64 << bit2;
    filter[idx3] |= 1u64 << bit3;

    already_present
}

/// Clear the bloom filter
#[inline(always)]
fn bloom_clear(filter: &mut [u64; 8]) {
    // Unroll for eBPF - avoid variable loop
    filter[0] = 0;
    filter[1] = 0;
    filter[2] = 0;
    filter[3] = 0;
    filter[4] = 0;
    filter[5] = 0;
    filter[6] = 0;
    filter[7] = 0;
}

#[inline(always)]
fn is_port_scan(src_ip: u32, dst_port: u16, now: u64, config: &UdpConfig) -> bool {
    let threshold = if config.portscan_threshold != 0 {
        config.portscan_threshold
    } else {
        DEFAULT_PORTSCAN_THRESHOLD
    };

    let window = if config.rate_limit_window_ns != 0 {
        config.rate_limit_window_ns
    } else {
        DEFAULT_RATE_LIMIT_WINDOW_NS
    };

    if let Some(state) = unsafe { UDP_IP_STATE_V4.get_ptr_mut(&src_ip) } {
        let state = unsafe { &mut *state };

        // Check if in new window - reset bloom filter
        if now.saturating_sub(state.window_start) > window {
            state.window_start = now;
            state.unique_ports = 0;
            bloom_clear(&mut state.port_bloom_filter);
            state.flags &= !FLAG_PORTSCAN_DETECTED;
        }

        // Check bloom filter - only increment if this is a new port
        let port_already_seen = bloom_check_and_add(&mut state.port_bloom_filter, dst_port);

        if !port_already_seen {
            // This is a genuinely new port (with high probability)
            state.unique_ports += 1;

            if state.unique_ports > threshold {
                state.flags |= FLAG_PORTSCAN_DETECTED;
                return true;
            }
        }
    }

    false
}

// ============================================================================
// Rate Limiting
// ============================================================================

#[inline(always)]
fn check_rate_limit_v4(src_ip: u32, bytes: u64, now: u64, config: &UdpConfig) -> bool {
    let window = if config.rate_limit_window_ns != 0 {
        config.rate_limit_window_ns
    } else {
        DEFAULT_RATE_LIMIT_WINDOW_NS
    };

    let max_packets = if config.max_packets_per_window != 0 {
        config.max_packets_per_window
    } else {
        DEFAULT_MAX_PACKETS_PER_WINDOW
    };

    let max_bytes = if config.max_bytes_per_window != 0 {
        config.max_bytes_per_window
    } else {
        DEFAULT_MAX_BYTES_PER_WINDOW
    };

    if let Some(state) = unsafe { UDP_IP_STATE_V4.get_ptr_mut(&src_ip) } {
        let state = unsafe { &mut *state };

        // Check if blocked
        if state.blocked_until > now {
            return false;
        }

        // Check if in new window
        if now.saturating_sub(state.window_start) > window {
            state.window_start = now;
            state.window_packets = 1;
            state.unique_ports = 1;
            state.packets += 1;
            state.bytes += bytes;
            state.last_seen = now;
            return true;
        }

        // Update counters
        state.window_packets += 1;
        state.packets += 1;
        state.bytes += bytes;
        state.last_seen = now;

        // Check limits
        if state.window_packets > max_packets || state.bytes > max_bytes {
            state.flags |= FLAG_FLOOD_DETECTED;
            state.blocked_until = now + config.block_duration_ns;
            return false;
        }

        true
    } else {
        // First packet from this IP
        let state = UdpIpState {
            packets: 1,
            bytes,
            window_start: now,
            window_packets: 1,
            last_seen: now,
            unique_ports: 1,
            amp_responses: 0,
            blocked_until: 0,
            flags: 0,
            port_bloom_filter: [0; 8],
        };
        let _ = UDP_IP_STATE_V4.insert(&src_ip, &state, 0);
        true
    }
}

#[inline(always)]
fn is_ip_blocked_v4(src_ip: u32) -> bool {
    if let Some(state) = unsafe { UDP_IP_STATE_V4.get(&src_ip) } {
        let now = unsafe { aya_ebpf::helpers::bpf_ktime_get_ns() };
        state.blocked_until > now
    } else {
        false
    }
}

#[inline(always)]
fn is_ip_blocked_v6(src_ip: &[u8; 16]) -> bool {
    if let Some(state) = unsafe { UDP_IP_STATE_V6.get(src_ip) } {
        let now = unsafe { aya_ebpf::helpers::bpf_ktime_get_ns() };
        state.blocked_until > now
    } else {
        false
    }
}

#[inline(always)]
fn block_ip_v4(src_ip: u32, duration_ns: u64) {
    let now = unsafe { aya_ebpf::helpers::bpf_ktime_get_ns() };
    let block_until = now
        + if duration_ns != 0 {
            duration_ns
        } else {
            DEFAULT_BLOCK_DURATION_NS
        };

    if let Some(state) = unsafe { UDP_IP_STATE_V4.get_ptr_mut(&src_ip) } {
        let state = unsafe { &mut *state };
        state.blocked_until = block_until;
    } else {
        let state = UdpIpState {
            packets: 0,
            bytes: 0,
            window_start: now,
            window_packets: 0,
            last_seen: now,
            unique_ports: 0,
            amp_responses: 0,
            blocked_until: block_until,
            flags: 0,
            port_bloom_filter: [0; 8],
        };
        let _ = UDP_IP_STATE_V4.insert(&src_ip, &state, 0);
    }
}

// ============================================================================
// IPv6 Rate Limiting and Port Scan Detection
// ============================================================================

#[inline(always)]
fn check_rate_limit_v6(src_ip: &[u8; 16], bytes: u64, now: u64, config: &UdpConfig) -> bool {
    let window = if config.rate_limit_window_ns != 0 {
        config.rate_limit_window_ns
    } else {
        DEFAULT_RATE_LIMIT_WINDOW_NS
    };

    let max_packets = if config.max_packets_per_window != 0 {
        config.max_packets_per_window
    } else {
        DEFAULT_MAX_PACKETS_PER_WINDOW
    };

    let max_bytes = if config.max_bytes_per_window != 0 {
        config.max_bytes_per_window
    } else {
        DEFAULT_MAX_BYTES_PER_WINDOW
    };

    if let Some(state) = unsafe { UDP_IP_STATE_V6.get_ptr_mut(src_ip) } {
        let state = unsafe { &mut *state };

        // Check if blocked
        if state.blocked_until > now {
            return false;
        }

        // Check if in new window
        if now.saturating_sub(state.window_start) > window {
            state.window_start = now;
            state.window_packets = 1;
            state.unique_ports = 1;
            state.packets += 1;
            state.bytes += bytes;
            state.last_seen = now;
            bloom_clear(&mut state.port_bloom_filter);
            return true;
        }

        // Update counters
        state.window_packets += 1;
        state.packets += 1;
        state.bytes += bytes;
        state.last_seen = now;

        // Check limits
        if state.window_packets > max_packets || state.bytes > max_bytes {
            state.flags |= FLAG_FLOOD_DETECTED;
            state.blocked_until = now + config.block_duration_ns;
            return false;
        }

        true
    } else {
        // First packet from this IPv6
        let state = UdpIpState {
            packets: 1,
            bytes,
            window_start: now,
            window_packets: 1,
            last_seen: now,
            unique_ports: 1,
            amp_responses: 0,
            blocked_until: 0,
            flags: 0,
            port_bloom_filter: [0; 8],
        };
        let _ = UDP_IP_STATE_V6.insert(src_ip, &state, 0);
        true
    }
}

#[inline(always)]
fn is_port_scan_v6(src_ip: &[u8; 16], dst_port: u16, now: u64, config: &UdpConfig) -> bool {
    let threshold = if config.portscan_threshold != 0 {
        config.portscan_threshold
    } else {
        DEFAULT_PORTSCAN_THRESHOLD
    };

    let window = if config.rate_limit_window_ns != 0 {
        config.rate_limit_window_ns
    } else {
        DEFAULT_RATE_LIMIT_WINDOW_NS
    };

    if let Some(state) = unsafe { UDP_IP_STATE_V6.get_ptr_mut(src_ip) } {
        let state = unsafe { &mut *state };

        // Check if in new window - reset bloom filter
        if now.saturating_sub(state.window_start) > window {
            state.window_start = now;
            state.unique_ports = 0;
            bloom_clear(&mut state.port_bloom_filter);
            state.flags &= !FLAG_PORTSCAN_DETECTED;
        }

        // Check bloom filter - only increment if this is a new port
        let port_already_seen = bloom_check_and_add(&mut state.port_bloom_filter, dst_port);

        if !port_already_seen {
            state.unique_ports += 1;

            if state.unique_ports > threshold {
                state.flags |= FLAG_PORTSCAN_DETECTED;
                return true;
            }
        }
    }

    false
}

#[inline(always)]
fn block_ip_v6(src_ip: &[u8; 16], duration_ns: u64) {
    let now = unsafe { aya_ebpf::helpers::bpf_ktime_get_ns() };
    let block_until = now
        + if duration_ns != 0 {
            duration_ns
        } else {
            DEFAULT_BLOCK_DURATION_NS
        };

    if let Some(state) = unsafe { UDP_IP_STATE_V6.get_ptr_mut(src_ip) } {
        let state = unsafe { &mut *state };
        state.blocked_until = block_until;
    } else {
        let state = UdpIpState {
            packets: 0,
            bytes: 0,
            window_start: now,
            window_packets: 0,
            last_seen: now,
            unique_ports: 0,
            amp_responses: 0,
            blocked_until: block_until,
            flags: 0,
            port_bloom_filter: [0; 8],
        };
        let _ = UDP_IP_STATE_V6.insert(src_ip, &state, 0);
    }
}

// ============================================================================
// Protocol Statistics
// ============================================================================

#[inline(always)]
fn track_protocol_stats(src_port: u16, dst_port: u16) {
    if let Some(stats) = unsafe { UDP_STATS.get_ptr_mut(0) } {
        let stats = unsafe { &mut *stats };

        match src_port {
            PORT_DNS => stats.dns_packets += 1,
            PORT_NTP => stats.ntp_packets += 1,
            PORT_SSDP => stats.ssdp_packets += 1,
            PORT_MEMCACHED => stats.memcached_packets += 1,
            _ => {}
        }

        // Also check destination port
        match dst_port {
            PORT_DNS => stats.dns_packets += 1,
            PORT_NTP => stats.ntp_packets += 1,
            PORT_SSDP => stats.ssdp_packets += 1,
            PORT_MEMCACHED => stats.memcached_packets += 1,
            _ => {}
        }
    }
}

// ============================================================================
// Configuration
// ============================================================================

#[inline(always)]
fn get_config() -> UdpConfig {
    if let Some(config) = unsafe { UDP_CONFIG.get_ptr(0) } {
        unsafe { *config }
    } else {
        UdpConfig {
            enabled: 1,
            min_packet_size: DEFAULT_MIN_PACKET_SIZE,
            max_packet_size: DEFAULT_MAX_PACKET_SIZE,
            rate_limit_window_ns: DEFAULT_RATE_LIMIT_WINDOW_NS,
            max_packets_per_window: DEFAULT_MAX_PACKETS_PER_WINDOW,
            max_bytes_per_window: DEFAULT_MAX_BYTES_PER_WINDOW,
            block_duration_ns: DEFAULT_BLOCK_DURATION_NS,
            protection_level: 2,
            amp_detection_enabled: 1,
            portscan_detection_enabled: 1,
            portscan_threshold: DEFAULT_PORTSCAN_THRESHOLD,
        }
    }
}

// ============================================================================
// Statistics
// ============================================================================

#[inline(always)]
fn update_stats_total() {
    if let Some(stats) = unsafe { UDP_STATS.get_ptr_mut(0) } {
        unsafe {
            (*stats).total_packets += 1;
        }
    }
}

#[inline(always)]
fn update_stats_passed() {
    if let Some(stats) = unsafe { UDP_STATS.get_ptr_mut(0) } {
        unsafe {
            (*stats).passed_packets += 1;
        }
    }
}

#[inline(always)]
fn update_stats_rate_limited() {
    if let Some(stats) = unsafe { UDP_STATS.get_ptr_mut(0) } {
        unsafe {
            (*stats).dropped_rate_limited += 1;
        }
    }
}

#[inline(always)]
fn update_stats_invalid_size() {
    if let Some(stats) = unsafe { UDP_STATS.get_ptr_mut(0) } {
        unsafe {
            (*stats).dropped_invalid_size += 1;
        }
    }
}

#[inline(always)]
fn update_stats_amplification() {
    if let Some(stats) = unsafe { UDP_STATS.get_ptr_mut(0) } {
        unsafe {
            (*stats).dropped_amplification += 1;
        }
    }
}

#[inline(always)]
fn update_stats_port_scan() {
    if let Some(stats) = unsafe { UDP_STATS.get_ptr_mut(0) } {
        unsafe {
            (*stats).dropped_port_scan += 1;
        }
    }
}

#[inline(always)]
fn update_stats_blocked() {
    if let Some(stats) = unsafe { UDP_STATS.get_ptr_mut(0) } {
        unsafe {
            (*stats).dropped_blocked_ip += 1;
        }
    }
}

#[inline(always)]
fn update_stats_blocked_port() {
    if let Some(stats) = unsafe { UDP_STATS.get_ptr_mut(0) } {
        unsafe {
            (*stats).dropped_blocked_port += 1;
        }
    }
}

#[inline(always)]
fn update_stats_fragmented() {
    if let Some(stats) = unsafe { UDP_STATS.get_ptr_mut(0) } {
        unsafe {
            (*stats).dropped_fragmented += 1;
        }
    }
}

// ============================================================================
// Panic Handler
// ============================================================================

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
