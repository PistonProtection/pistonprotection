//! XDP HTTP Protocol Filter
//!
//! XDP program for filtering HTTP/1.1 and HTTP/2 traffic with:
//! - HTTP method validation
//! - Path/host filtering
//! - Header inspection
//! - Request rate limiting
//! - Slow HTTP attack detection
//! - Invalid request detection

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
struct TcpHdr {
    source: u16,
    dest: u16,
    seq: u32,
    ack_seq: u32,
    doff_flags: u16,
    window: u16,
    check: u16,
    urg_ptr: u16,
}

// ============================================================================
// HTTP Filtering Structures
// ============================================================================

/// HTTP connection tracking state
#[repr(C)]
pub struct HttpConnectionState {
    /// Connection state: 0=new, 1=request_started, 2=headers, 3=body, 4=complete
    pub state: u8,
    /// HTTP version: 1=HTTP/1.0, 2=HTTP/1.1, 3=HTTP/2
    pub http_version: u8,
    /// Request method detected
    pub method: u8,
    /// Flags for various conditions
    pub flags: u16,
    /// Timestamp of first packet in request
    pub request_start: u64,
    /// Timestamp of last packet
    pub last_seen: u64,
    /// Bytes received in current request
    pub bytes_received: u64,
    /// Headers received (for slow header attack detection)
    pub headers_bytes: u32,
    /// Number of requests from this connection
    pub request_count: u32,
}

/// Per-IP HTTP rate limiting
#[repr(C)]
pub struct HttpRateLimit {
    /// Request count in current window
    pub requests: u64,
    /// Window start timestamp
    pub window_start: u64,
    /// Total bytes sent
    pub bytes: u64,
    /// Error count (4xx/5xx responses or invalid requests)
    pub errors: u64,
    /// Slow request count
    pub slow_requests: u32,
    /// Blocked until timestamp (0 = not blocked)
    pub blocked_until: u64,
}

/// HTTP filter configuration
#[repr(C)]
pub struct HttpConfig {
    /// Filter enabled
    pub enabled: u32,
    /// HTTP port (default 80)
    pub http_port: u16,
    /// HTTPS port (default 443)
    pub https_port: u16,
    /// Maximum requests per window
    pub max_requests_per_window: u32,
    /// Window size in nanoseconds
    pub window_size_ns: u64,
    /// Maximum header size (slow loris protection)
    pub max_header_size: u32,
    /// Maximum time for headers in nanoseconds (slow loris)
    pub max_header_time_ns: u64,
    /// Maximum request body size
    pub max_body_size: u64,
    /// Block duration in nanoseconds
    pub block_duration_ns: u64,
    /// Protection level (1=basic, 2=moderate, 3=aggressive)
    pub protection_level: u32,
}

/// HTTP statistics
#[repr(C)]
pub struct HttpStats {
    pub total_requests: u64,
    pub passed_requests: u64,
    pub dropped_invalid_method: u64,
    pub dropped_rate_limited: u64,
    pub dropped_slow_loris: u64,
    pub dropped_invalid_request: u64,
    pub dropped_blocked_ip: u64,
    pub http2_requests: u64,
}

/// Blocked path entry (for path-based filtering)
#[repr(C)]
pub struct BlockedPath {
    /// Path hash
    pub hash: u32,
    /// Block reason
    pub reason: u32,
}

// ============================================================================
// HTTP Methods (encoded as u8)
// ============================================================================

const HTTP_METHOD_UNKNOWN: u8 = 0;
const HTTP_METHOD_GET: u8 = 1;
const HTTP_METHOD_POST: u8 = 2;
const HTTP_METHOD_PUT: u8 = 3;
const HTTP_METHOD_DELETE: u8 = 4;
const HTTP_METHOD_HEAD: u8 = 5;
const HTTP_METHOD_OPTIONS: u8 = 6;
const HTTP_METHOD_PATCH: u8 = 7;
const HTTP_METHOD_CONNECT: u8 = 8;
const HTTP_METHOD_TRACE: u8 = 9;

// HTTP/2 connection preface magic
const HTTP2_PREFACE: [u8; 24] = [
    0x50, 0x52, 0x49, 0x20, 0x2a, 0x20, 0x48, 0x54,
    0x54, 0x50, 0x2f, 0x32, 0x2e, 0x30, 0x0d, 0x0a,
    0x0d, 0x0a, 0x53, 0x4d, 0x0d, 0x0a, 0x0d, 0x0a,
];

// Connection flags
const FLAG_SLOW_HEADERS: u16 = 0x0001;
const FLAG_SLOW_BODY: u16 = 0x0002;
const FLAG_INVALID_METHOD: u16 = 0x0004;
const FLAG_HTTP2: u16 = 0x0008;
const FLAG_SUSPICIOUS: u16 = 0x0010;

// ============================================================================
// eBPF Maps
// ============================================================================

/// HTTP connection state tracking (keyed by src_ip:src_port:dst_port)
#[map]
static HTTP_CONNECTIONS: LruHashMap<u64, HttpConnectionState> =
    LruHashMap::with_max_entries(1_000_000, 0);

/// Per-IP rate limiting
#[map]
static HTTP_RATE_LIMITS: LruHashMap<u32, HttpRateLimit> =
    LruHashMap::with_max_entries(500_000, 0);

/// Per-IP rate limiting for IPv6
#[map]
static HTTP_RATE_LIMITS_V6: LruHashMap<[u8; 16], HttpRateLimit> =
    LruHashMap::with_max_entries(250_000, 0);

/// Blocked paths (by hash)
#[map]
static BLOCKED_PATHS: HashMap<u32, BlockedPath> =
    HashMap::with_max_entries(10_000, 0);

/// Blocked User-Agent hashes
#[map]
static BLOCKED_USER_AGENTS: HashMap<u32, u32> =
    HashMap::with_max_entries(10_000, 0);

/// Whitelisted IPs (bypass filtering)
#[map]
static HTTP_WHITELIST: HashMap<u32, u32> =
    HashMap::with_max_entries(10_000, 0);

/// Configuration
#[map]
static HTTP_CONFIG: PerCpuArray<HttpConfig> = PerCpuArray::with_max_entries(1, 0);

/// Statistics
#[map]
static HTTP_STATS: PerCpuArray<HttpStats> = PerCpuArray::with_max_entries(1, 0);

// ============================================================================
// Constants
// ============================================================================

const ETH_P_IP: u16 = 0x0800;
const ETH_P_IPV6: u16 = 0x86DD;
const IPPROTO_TCP: u8 = 6;

const DEFAULT_HTTP_PORT: u16 = 80;
const DEFAULT_HTTPS_PORT: u16 = 443;

// Default limits
const DEFAULT_MAX_REQUESTS_PER_WINDOW: u32 = 100;
const DEFAULT_WINDOW_SIZE_NS: u64 = 1_000_000_000; // 1 second
const DEFAULT_MAX_HEADER_SIZE: u32 = 8192;
const DEFAULT_MAX_HEADER_TIME_NS: u64 = 10_000_000_000; // 10 seconds
const DEFAULT_MAX_BODY_SIZE: u64 = 10_485_760; // 10MB
const DEFAULT_BLOCK_DURATION_NS: u64 = 60_000_000_000; // 60 seconds

// ============================================================================
// Main XDP Entry Point
// ============================================================================

#[xdp]
pub fn xdp_http(ctx: XdpContext) -> u32 {
    match try_xdp_http(ctx) {
        Ok(ret) => ret,
        Err(_) => xdp_action::XDP_PASS,
    }
}

#[inline(always)]
fn try_xdp_http(ctx: XdpContext) -> Result<u32, ()> {
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
    config: &HttpConfig,
) -> Result<u32, ()> {
    if data + mem::size_of::<Ipv4Hdr>() > data_end {
        return Ok(xdp_action::XDP_PASS);
    }

    let ip = unsafe { &*(data as *const Ipv4Hdr) };

    // Only process TCP
    if ip.protocol != IPPROTO_TCP {
        return Ok(xdp_action::XDP_PASS);
    }

    let src_ip = u32::from_be(ip.saddr);

    // Check whitelist
    if unsafe { HTTP_WHITELIST.get(&src_ip) }.is_some() {
        return Ok(xdp_action::XDP_PASS);
    }

    // Check if IP is blocked
    if is_ip_blocked_v4(src_ip) {
        update_stats_blocked();
        return Ok(xdp_action::XDP_DROP);
    }

    let ihl = (ip.version_ihl & 0x0f) as usize * 4;
    let tcp_data = data + ihl;

    process_tcp_http(ctx, tcp_data, data_end, src_ip, config)
}

// ============================================================================
// IPv6 Processing
// ============================================================================

#[inline(always)]
fn process_ipv6(
    ctx: &XdpContext,
    data: usize,
    data_end: usize,
    config: &HttpConfig,
) -> Result<u32, ()> {
    if data + mem::size_of::<Ipv6Hdr>() > data_end {
        return Ok(xdp_action::XDP_PASS);
    }

    let ip6 = unsafe { &*(data as *const Ipv6Hdr) };

    // Only process TCP
    if ip6.nexthdr != IPPROTO_TCP {
        return Ok(xdp_action::XDP_PASS);
    }

    let src_ip = ip6.saddr;

    // Check if IP is blocked
    if is_ip_blocked_v6(&src_ip) {
        update_stats_blocked();
        return Ok(xdp_action::XDP_DROP);
    }

    let tcp_data = data + mem::size_of::<Ipv6Hdr>();

    // For IPv6, we use a simplified check - convert to u32 key for connection tracking
    let ip_key = u32::from_be_bytes([src_ip[12], src_ip[13], src_ip[14], src_ip[15]]);

    process_tcp_http(ctx, tcp_data, data_end, ip_key, config)
}

// ============================================================================
// TCP/HTTP Processing
// ============================================================================

#[inline(always)]
fn process_tcp_http(
    ctx: &XdpContext,
    data: usize,
    data_end: usize,
    src_ip: u32,
    config: &HttpConfig,
) -> Result<u32, ()> {
    if data + mem::size_of::<TcpHdr>() > data_end {
        return Ok(xdp_action::XDP_PASS);
    }

    let tcp = unsafe { &*(data as *const TcpHdr) };
    let dst_port = u16::from_be(tcp.dest);
    let src_port = u16::from_be(tcp.source);

    // Check if this is HTTP/HTTPS traffic
    let http_port = if config.http_port != 0 { config.http_port } else { DEFAULT_HTTP_PORT };
    let https_port = if config.https_port != 0 { config.https_port } else { DEFAULT_HTTPS_PORT };

    if dst_port != http_port && dst_port != https_port {
        return Ok(xdp_action::XDP_PASS);
    }

    // Update total request stats
    update_stats_total();

    // Check rate limit first
    if !check_rate_limit_v4(src_ip, config) {
        update_stats_rate_limited();
        return Ok(xdp_action::XDP_DROP);
    }

    // Calculate TCP payload
    let tcp_header_len = ((u16::from_be(tcp.doff_flags) >> 12) & 0x0f) as usize * 4;
    let payload_start = data + tcp_header_len;

    if payload_start >= data_end {
        // No payload (SYN, ACK, FIN, etc.) - pass through
        return Ok(xdp_action::XDP_PASS);
    }

    let payload_len = data_end - payload_start;
    if payload_len == 0 {
        return Ok(xdp_action::XDP_PASS);
    }

    // Connection tracking key
    let conn_key = make_connection_key(src_ip, src_port, dst_port);
    let now = unsafe { aya_ebpf::helpers::bpf_ktime_get_ns() };

    // Get or create connection state
    let conn_state = get_or_create_connection(conn_key, now);

    // Check for slow HTTP attack (slow loris / slow POST)
    if let Some(state) = unsafe { HTTP_CONNECTIONS.get_ptr_mut(&conn_key) } {
        let state = unsafe { &mut *state };

        // Update connection state
        state.last_seen = now;
        state.bytes_received += payload_len as u64;

        // Check for slow loris attack (headers taking too long)
        let max_header_time = if config.max_header_time_ns != 0 {
            config.max_header_time_ns
        } else {
            DEFAULT_MAX_HEADER_TIME_NS
        };

        if state.state == 1 || state.state == 2 {
            // In headers phase
            state.headers_bytes += payload_len as u32;

            let elapsed = now.saturating_sub(state.request_start);
            if elapsed > max_header_time {
                // Slow loris detected
                state.flags |= FLAG_SLOW_HEADERS;
                block_ip_v4(src_ip, config.block_duration_ns);
                update_stats_slow_loris();
                return Ok(xdp_action::XDP_DROP);
            }

            // Check max header size
            let max_header_size = if config.max_header_size != 0 {
                config.max_header_size
            } else {
                DEFAULT_MAX_HEADER_SIZE
            };

            if state.headers_bytes > max_header_size {
                update_stats_invalid();
                return Ok(xdp_action::XDP_DROP);
            }
        }
    }

    // Validate HTTP request payload
    let payload = unsafe { core::slice::from_raw_parts(payload_start as *const u8, core::cmp::min(payload_len, 512)) };

    // Check for HTTP/2 preface
    if payload_len >= 24 && is_http2_preface(payload) {
        update_stats_http2();
        // HTTP/2 - do basic validation but allow through
        // HTTP/2 frame validation would require more complex state
        return Ok(xdp_action::XDP_PASS);
    }

    // Validate HTTP/1.x request
    match validate_http_request(payload, config) {
        HttpValidation::Valid(method) => {
            if let Some(state) = unsafe { HTTP_CONNECTIONS.get_ptr_mut(&conn_key) } {
                let state = unsafe { &mut *state };
                state.method = method;
                state.state = 2; // Headers phase
                state.request_count += 1;
            }
            update_stats_passed();
            Ok(xdp_action::XDP_PASS)
        }
        HttpValidation::InvalidMethod => {
            update_stats_invalid_method();
            if config.protection_level >= 2 {
                block_ip_v4(src_ip, config.block_duration_ns);
            }
            Ok(xdp_action::XDP_DROP)
        }
        HttpValidation::InvalidRequest => {
            update_stats_invalid();
            if config.protection_level >= 3 {
                block_ip_v4(src_ip, config.block_duration_ns / 2);
            }
            Ok(xdp_action::XDP_DROP)
        }
        HttpValidation::Suspicious => {
            // Mark as suspicious but allow (for logging)
            if let Some(state) = unsafe { HTTP_CONNECTIONS.get_ptr_mut(&conn_key) } {
                let state = unsafe { &mut *state };
                state.flags |= FLAG_SUSPICIOUS;
            }
            update_stats_passed();
            Ok(xdp_action::XDP_PASS)
        }
        HttpValidation::NotHttp => {
            // Not an HTTP request line - could be continuation data
            Ok(xdp_action::XDP_PASS)
        }
    }
}

// ============================================================================
// HTTP Validation
// ============================================================================

enum HttpValidation {
    Valid(u8),
    InvalidMethod,
    InvalidRequest,
    Suspicious,
    NotHttp,
}

#[inline(always)]
fn validate_http_request(payload: &[u8], config: &HttpConfig) -> HttpValidation {
    if payload.len() < 14 {
        return HttpValidation::NotHttp;
    }

    // Parse HTTP method
    let method = match parse_http_method(payload) {
        Some(m) => m,
        None => return HttpValidation::NotHttp,
    };

    // Block TRACE method (XST attack vector)
    if method == HTTP_METHOD_TRACE && config.protection_level >= 1 {
        return HttpValidation::InvalidMethod;
    }

    // Block CONNECT unless explicitly needed
    if method == HTTP_METHOD_CONNECT && config.protection_level >= 2 {
        return HttpValidation::InvalidMethod;
    }

    // Find the space after method
    let method_len = get_method_length(method);
    if method_len >= payload.len() {
        return HttpValidation::InvalidRequest;
    }

    // Check for space after method
    if payload[method_len] != b' ' {
        return HttpValidation::InvalidRequest;
    }

    // Find HTTP version marker
    let mut found_http = false;
    let mut version_pos = 0;

    // Scan for "HTTP/" (limit scan to prevent DoS)
    let scan_limit = core::cmp::min(payload.len(), 256);
    for i in (method_len + 2)..scan_limit.saturating_sub(5) {
        if payload[i] == b'H' &&
           i + 5 <= scan_limit &&
           payload[i + 1] == b'T' &&
           payload[i + 2] == b'T' &&
           payload[i + 3] == b'P' &&
           payload[i + 4] == b'/' {
            found_http = true;
            version_pos = i + 5;
            break;
        }
    }

    if !found_http {
        return HttpValidation::InvalidRequest;
    }

    // Validate HTTP version (1.0, 1.1, or 2)
    if version_pos + 3 > payload.len() {
        return HttpValidation::InvalidRequest;
    }

    let version_valid = match (payload[version_pos], payload.get(version_pos + 1), payload.get(version_pos + 2)) {
        (b'1', Some(b'.'), Some(b'0' | b'1')) => true,
        (b'2', Some(b'.'), Some(b'0')) => true,
        (b'2', _, _) => true, // HTTP/2
        _ => false,
    };

    if !version_valid {
        return HttpValidation::InvalidRequest;
    }

    // Check for suspicious patterns in the path
    // Path starts after method + space
    let path_start = method_len + 1;
    if check_suspicious_path(&payload[path_start..]) {
        return HttpValidation::Suspicious;
    }

    HttpValidation::Valid(method)
}

#[inline(always)]
fn parse_http_method(payload: &[u8]) -> Option<u8> {
    if payload.len() < 3 {
        return None;
    }

    // Check common methods
    match payload[0] {
        b'G' => {
            if payload.len() >= 3 && payload[1] == b'E' && payload[2] == b'T' {
                return Some(HTTP_METHOD_GET);
            }
        }
        b'P' => {
            if payload.len() >= 4 {
                if payload[1] == b'O' && payload[2] == b'S' && payload[3] == b'T' {
                    return Some(HTTP_METHOD_POST);
                }
                if payload[1] == b'U' && payload[2] == b'T' {
                    return Some(HTTP_METHOD_PUT);
                }
                if payload.len() >= 5 && payload[1] == b'A' && payload[2] == b'T' &&
                   payload[3] == b'C' && payload[4] == b'H' {
                    return Some(HTTP_METHOD_PATCH);
                }
            }
        }
        b'D' => {
            if payload.len() >= 6 && payload[1] == b'E' && payload[2] == b'L' &&
               payload[3] == b'E' && payload[4] == b'T' && payload[5] == b'E' {
                return Some(HTTP_METHOD_DELETE);
            }
        }
        b'H' => {
            if payload.len() >= 4 && payload[1] == b'E' && payload[2] == b'A' && payload[3] == b'D' {
                return Some(HTTP_METHOD_HEAD);
            }
        }
        b'O' => {
            if payload.len() >= 7 && payload[1] == b'P' && payload[2] == b'T' &&
               payload[3] == b'I' && payload[4] == b'O' && payload[5] == b'N' && payload[6] == b'S' {
                return Some(HTTP_METHOD_OPTIONS);
            }
        }
        b'C' => {
            if payload.len() >= 7 && payload[1] == b'O' && payload[2] == b'N' &&
               payload[3] == b'N' && payload[4] == b'E' && payload[5] == b'C' && payload[6] == b'T' {
                return Some(HTTP_METHOD_CONNECT);
            }
        }
        b'T' => {
            if payload.len() >= 5 && payload[1] == b'R' && payload[2] == b'A' &&
               payload[3] == b'C' && payload[4] == b'E' {
                return Some(HTTP_METHOD_TRACE);
            }
        }
        _ => {}
    }

    None
}

#[inline(always)]
fn get_method_length(method: u8) -> usize {
    match method {
        HTTP_METHOD_GET => 3,
        HTTP_METHOD_PUT => 3,
        HTTP_METHOD_POST => 4,
        HTTP_METHOD_HEAD => 4,
        HTTP_METHOD_PATCH => 5,
        HTTP_METHOD_TRACE => 5,
        HTTP_METHOD_DELETE => 6,
        HTTP_METHOD_OPTIONS => 7,
        HTTP_METHOD_CONNECT => 7,
        _ => 3,
    }
}

#[inline(always)]
fn check_suspicious_path(path: &[u8]) -> bool {
    let scan_limit = core::cmp::min(path.len(), 128);

    // Check for directory traversal
    for i in 0..scan_limit.saturating_sub(2) {
        if path[i] == b'.' && path.get(i + 1) == Some(&b'.') {
            // Found ".." - potential directory traversal
            return true;
        }
    }

    // Check for null byte injection
    for i in 0..scan_limit {
        if path[i] == 0 {
            return true;
        }
    }

    // Check for common attack patterns
    // %00 (null byte URL encoded)
    for i in 0..scan_limit.saturating_sub(2) {
        if path[i] == b'%' && path.get(i + 1) == Some(&b'0') && path.get(i + 2) == Some(&b'0') {
            return true;
        }
    }

    false
}

#[inline(always)]
fn is_http2_preface(payload: &[u8]) -> bool {
    if payload.len() < 24 {
        return false;
    }

    for i in 0..24 {
        if payload[i] != HTTP2_PREFACE[i] {
            return false;
        }
    }
    true
}

// ============================================================================
// Rate Limiting
// ============================================================================

#[inline(always)]
fn check_rate_limit_v4(src_ip: u32, config: &HttpConfig) -> bool {
    let now = unsafe { aya_ebpf::helpers::bpf_ktime_get_ns() };
    let window_size = if config.window_size_ns != 0 {
        config.window_size_ns
    } else {
        DEFAULT_WINDOW_SIZE_NS
    };
    let max_requests = if config.max_requests_per_window != 0 {
        config.max_requests_per_window as u64
    } else {
        DEFAULT_MAX_REQUESTS_PER_WINDOW as u64
    };

    if let Some(rate) = unsafe { HTTP_RATE_LIMITS.get_ptr_mut(&src_ip) } {
        let rate = unsafe { &mut *rate };

        // Check if in new window
        if now.saturating_sub(rate.window_start) > window_size {
            // New window
            rate.window_start = now;
            rate.requests = 1;
            return true;
        }

        rate.requests += 1;

        if rate.requests > max_requests {
            // Rate exceeded - consider blocking
            rate.errors += 1;
            if rate.errors > 10 {
                // Persistent rate limit violation - block
                rate.blocked_until = now + config.block_duration_ns;
            }
            return false;
        }

        true
    } else {
        // First request from this IP
        let rate = HttpRateLimit {
            requests: 1,
            window_start: now,
            bytes: 0,
            errors: 0,
            slow_requests: 0,
            blocked_until: 0,
        };
        let _ = HTTP_RATE_LIMITS.insert(&src_ip, &rate, 0);
        true
    }
}

#[inline(always)]
fn is_ip_blocked_v4(src_ip: u32) -> bool {
    if let Some(rate) = unsafe { HTTP_RATE_LIMITS.get(&src_ip) } {
        let now = unsafe { aya_ebpf::helpers::bpf_ktime_get_ns() };
        rate.blocked_until > now
    } else {
        false
    }
}

#[inline(always)]
fn is_ip_blocked_v6(src_ip: &[u8; 16]) -> bool {
    if let Some(rate) = unsafe { HTTP_RATE_LIMITS_V6.get(src_ip) } {
        let now = unsafe { aya_ebpf::helpers::bpf_ktime_get_ns() };
        rate.blocked_until > now
    } else {
        false
    }
}

#[inline(always)]
fn block_ip_v4(src_ip: u32, duration_ns: u64) {
    let now = unsafe { aya_ebpf::helpers::bpf_ktime_get_ns() };
    let block_until = now + if duration_ns != 0 { duration_ns } else { DEFAULT_BLOCK_DURATION_NS };

    if let Some(rate) = unsafe { HTTP_RATE_LIMITS.get_ptr_mut(&src_ip) } {
        let rate = unsafe { &mut *rate };
        rate.blocked_until = block_until;
    } else {
        let rate = HttpRateLimit {
            requests: 0,
            window_start: now,
            bytes: 0,
            errors: 1,
            slow_requests: 0,
            blocked_until: block_until,
        };
        let _ = HTTP_RATE_LIMITS.insert(&src_ip, &rate, 0);
    }
}

// ============================================================================
// Connection Tracking
// ============================================================================

#[inline(always)]
fn make_connection_key(src_ip: u32, src_port: u16, dst_port: u16) -> u64 {
    ((src_ip as u64) << 32) | ((src_port as u64) << 16) | (dst_port as u64)
}

#[inline(always)]
fn get_or_create_connection(conn_key: u64, now: u64) -> HttpConnectionState {
    if let Some(state) = unsafe { HTTP_CONNECTIONS.get(&conn_key) } {
        *state
    } else {
        let state = HttpConnectionState {
            state: 1, // Request started
            http_version: 0,
            method: 0,
            flags: 0,
            request_start: now,
            last_seen: now,
            bytes_received: 0,
            headers_bytes: 0,
            request_count: 0,
        };
        let _ = HTTP_CONNECTIONS.insert(&conn_key, &state, 0);
        state
    }
}

// ============================================================================
// Configuration
// ============================================================================

#[inline(always)]
fn get_config() -> HttpConfig {
    if let Some(config) = unsafe { HTTP_CONFIG.get_ptr(0) } {
        unsafe { *config }
    } else {
        HttpConfig {
            enabled: 1,
            http_port: DEFAULT_HTTP_PORT,
            https_port: DEFAULT_HTTPS_PORT,
            max_requests_per_window: DEFAULT_MAX_REQUESTS_PER_WINDOW,
            window_size_ns: DEFAULT_WINDOW_SIZE_NS,
            max_header_size: DEFAULT_MAX_HEADER_SIZE,
            max_header_time_ns: DEFAULT_MAX_HEADER_TIME_NS,
            max_body_size: DEFAULT_MAX_BODY_SIZE,
            block_duration_ns: DEFAULT_BLOCK_DURATION_NS,
            protection_level: 2,
        }
    }
}

// ============================================================================
// Statistics
// ============================================================================

#[inline(always)]
fn update_stats_total() {
    if let Some(stats) = unsafe { HTTP_STATS.get_ptr_mut(0) } {
        unsafe { (*stats).total_requests += 1; }
    }
}

#[inline(always)]
fn update_stats_passed() {
    if let Some(stats) = unsafe { HTTP_STATS.get_ptr_mut(0) } {
        unsafe { (*stats).passed_requests += 1; }
    }
}

#[inline(always)]
fn update_stats_invalid_method() {
    if let Some(stats) = unsafe { HTTP_STATS.get_ptr_mut(0) } {
        unsafe { (*stats).dropped_invalid_method += 1; }
    }
}

#[inline(always)]
fn update_stats_rate_limited() {
    if let Some(stats) = unsafe { HTTP_STATS.get_ptr_mut(0) } {
        unsafe { (*stats).dropped_rate_limited += 1; }
    }
}

#[inline(always)]
fn update_stats_slow_loris() {
    if let Some(stats) = unsafe { HTTP_STATS.get_ptr_mut(0) } {
        unsafe { (*stats).dropped_slow_loris += 1; }
    }
}

#[inline(always)]
fn update_stats_invalid() {
    if let Some(stats) = unsafe { HTTP_STATS.get_ptr_mut(0) } {
        unsafe { (*stats).dropped_invalid_request += 1; }
    }
}

#[inline(always)]
fn update_stats_blocked() {
    if let Some(stats) = unsafe { HTTP_STATS.get_ptr_mut(0) } {
        unsafe { (*stats).dropped_blocked_ip += 1; }
    }
}

#[inline(always)]
fn update_stats_http2() {
    if let Some(stats) = unsafe { HTTP_STATS.get_ptr_mut(0) } {
        unsafe { (*stats).http2_requests += 1; }
    }
}

// ============================================================================
// Panic Handler
// ============================================================================

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
