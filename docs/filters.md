# Protocol Filter Documentation

This document provides comprehensive documentation for PistonProtection's protocol filtering capabilities, including TCP/UDP filtering, HTTP/HTTPS filtering, QUIC filtering, Minecraft protocol filtering, and custom filter rules.

## Table of Contents

- [Overview](#overview)
- [Layer 4 Filtering](#layer-4-filtering)
  - [TCP Filtering](#tcp-filtering)
  - [UDP Filtering](#udp-filtering)
  - [ICMP Filtering](#icmp-filtering)
- [Layer 7 Filtering](#layer-7-filtering)
  - [HTTP/HTTPS Filtering](#httphttps-filtering)
  - [QUIC/HTTP3 Filtering](#quichttp3-filtering)
  - [Minecraft Java Edition](#minecraft-java-edition)
  - [Minecraft Bedrock Edition](#minecraft-bedrock-edition)
- [GeoIP Blocking](#geoip-blocking)
- [Rate Limiting](#rate-limiting)
- [Custom Filter Rules](#custom-filter-rules)
- [Filter Rule Examples](#filter-rule-examples)

---

## Overview

PistonProtection uses eBPF/XDP technology to perform line-rate packet filtering at the network driver level. This enables filtering millions of packets per second with minimal CPU overhead.

### Filter Processing Pipeline

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Incoming Packet                                   │
└─────────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────────┐
│  1. XDP Layer (Hardware/Driver Level)                                │
│     - IP validation                                                  │
│     - Blocked IP check                                               │
│     - Rate limiting (PPS)                                            │
│     - SYN cookie validation                                          │
└─────────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────────┐
│  2. Protocol Filter Layer                                            │
│     - TCP/UDP/ICMP validation                                        │
│     - Port matching                                                  │
│     - Connection tracking                                            │
└─────────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────────┐
│  3. L7 Protocol Filter Layer                                         │
│     - HTTP/HTTPS inspection                                          │
│     - QUIC validation                                                │
│     - Minecraft protocol validation                                  │
└─────────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────────┐
│  4. Custom Rules Layer                                               │
│     - User-defined filter rules                                      │
│     - GeoIP filtering                                                │
│     - ASN filtering                                                  │
└─────────────────────────────────────────────────────────────────────┘
                                │
                                ▼
                    ┌───────────────────┐
                    │  PASS / DROP      │
                    └───────────────────┘
```

### Filter Actions

| Action | Description | Use Case |
|--------|-------------|----------|
| `ALLOW` | Pass traffic through, bypass other rules | Whitelist trusted IPs |
| `DROP` | Silently drop packets | Block malicious traffic |
| `RATE_LIMIT` | Apply rate limiting | Limit per-IP traffic |
| `CHALLENGE` | Send L7 challenge | Verify human users |
| `LOG` | Log and pass through | Monitoring/debugging |
| `REDIRECT` | Redirect to honeypot | Deception/analysis |

---

## Layer 4 Filtering

### TCP Filtering

TCP filtering provides protection against TCP-based attacks.

#### SYN Flood Protection

PistonProtection implements SYN cookies at the XDP level:

```yaml
# Enable SYN flood protection
protection:
  protocols:
    synCookies:
      enabled: true
      # Threshold to activate SYN cookies (SYN packets/second)
      threshold: 10000
```

**How it works:**
1. When SYN rate exceeds threshold, XDP generates SYN cookies
2. Valid ACKs with correct cookies are allowed
3. Invalid connections are dropped without state

#### TCP Connection Tracking

```yaml
# Connection tracking settings
worker:
  ebpf:
    maps:
      connections:
        maxEntries: 1000000  # Max tracked connections
```

States tracked:
- `NEW` - Initial SYN received
- `ESTABLISHED` - Three-way handshake complete
- `RELATED` - Related to established connection
- `CLOSING` - FIN received
- `CLOSED` - Connection terminated

#### TCP Flag Filtering

Filter based on TCP flags:

```yaml
apiVersion: pistonprotection.io/v1
kind: FilterRule
metadata:
  name: block-invalid-tcp-flags
spec:
  match:
    protocols: [TCP]
    tcp:
      # Block Christmas tree packets
      flagsSet: [FIN, PSH, URG]
      flagsUnset: [ACK]
  action: DROP
```

#### TCP Port Filtering

```yaml
apiVersion: pistonprotection.io/v1
kind: FilterRule
metadata:
  name: allow-web-ports
spec:
  match:
    protocols: [TCP]
    destinationPorts:
      - start: 80
        end: 80
      - start: 443
        end: 443
  action: ALLOW
```

---

### UDP Filtering

UDP filtering protects against UDP-based attacks including amplification attacks.

#### UDP Amplification Protection

```yaml
# Protection against common amplification vectors
protection:
  protocols:
    udpAmplification:
      enabled: true
      # Block known amplification ports from untrusted sources
      blockedPorts:
        - 17    # QOTD
        - 19    # Chargen
        - 53    # DNS (rate limited)
        - 111   # RPC
        - 123   # NTP
        - 137   # NetBIOS
        - 161   # SNMP
        - 389   # LDAP
        - 1900  # SSDP
        - 11211 # Memcached
```

#### UDP Rate Limiting

```yaml
apiVersion: pistonprotection.io/v1
kind: FilterRule
metadata:
  name: rate-limit-udp
spec:
  match:
    protocols: [UDP]
  action: RATE_LIMIT
  rateLimit:
    requestsPerSecond: 1000
    burstSize: 2000
    windowSeconds: 60
```

#### UDP Response Ratio

Detect and mitigate UDP amplification by tracking request/response ratios:

```yaml
protection:
  protocols:
    udp:
      # Drop if response:request ratio exceeds threshold
      maxAmplificationRatio: 10
      # Time window for ratio calculation
      ratioWindowSeconds: 60
```

---

### ICMP Filtering

Control ICMP traffic to prevent ICMP floods and information leakage.

```yaml
apiVersion: pistonprotection.io/v1
kind: FilterRule
metadata:
  name: icmp-rate-limit
spec:
  match:
    protocols: [ICMP]
    icmp:
      # Allow only Echo Request and Echo Reply
      types: [0, 8]
  action: RATE_LIMIT
  rateLimit:
    requestsPerSecond: 10
    burstSize: 20
```

---

## Layer 7 Filtering

### HTTP/HTTPS Filtering

HTTP filtering provides deep inspection of HTTP traffic.

#### HTTP Method Filtering

```yaml
apiVersion: pistonprotection.io/v1
kind: FilterRule
metadata:
  name: http-method-filter
spec:
  match:
    l7Protocols: [HTTP, HTTP2, HTTP3]
    l7Match:
      http:
        methods: [GET, POST, HEAD, OPTIONS]
        # Block everything else
  action: ALLOW
---
apiVersion: pistonprotection.io/v1
kind: FilterRule
metadata:
  name: block-other-methods
spec:
  match:
    l7Protocols: [HTTP, HTTP2, HTTP3]
  action: DROP
  priority: 1000  # Lower priority
```

#### Path-Based Filtering

```yaml
apiVersion: pistonprotection.io/v1
kind: FilterRule
metadata:
  name: protect-admin-path
spec:
  match:
    l7Protocols: [HTTP, HTTP2]
    l7Match:
      http:
        paths:
          - "/admin/*"
          - "/wp-admin/*"
          - "/.env"
          - "/config/*"
  action: DROP
```

#### Header-Based Filtering

```yaml
apiVersion: pistonprotection.io/v1
kind: FilterRule
metadata:
  name: block-bad-user-agents
spec:
  match:
    l7Protocols: [HTTP, HTTP2]
    l7Match:
      http:
        userAgents:
          - "*sqlmap*"
          - "*nikto*"
          - "*nmap*"
          - "*masscan*"
          - "*python-requests*"
  action: DROP
```

#### Request Size Limits

```yaml
protection:
  l7Settings:
    http:
      maxRequestBodySize: 10485760  # 10MB
      maxHeaderSize: 8192           # 8KB
      maxRequestsPerConnection: 1000
      idleTimeout: 60s
```

#### WAF Rules

Enable Web Application Firewall:

```yaml
protection:
  l7Settings:
    http:
      wafEnabled: true
      wafMode: BLOCK  # DETECT or BLOCK
      # OWASP CRS ruleset
      wafRules:
        - sql-injection
        - xss
        - lfi-rfi
        - protocol-anomaly
        - request-smuggling
```

#### Bot Protection

```yaml
protection:
  l7Settings:
    http:
      botProtection: true
      botMode: CHALLENGE_SUSPICIOUS
      # Known good bots to allow
      allowedBots:
        - googlebot
        - bingbot
        - facebookexternalhit
```

#### TLS Requirements

```yaml
apiVersion: pistonprotection.io/v1
kind: FilterRule
metadata:
  name: require-tls
spec:
  match:
    l7Protocols: [HTTP]
    l7Match:
      http:
        requireTls: false  # Match non-TLS
  action: REDIRECT
  redirectUrl: "https://{host}{path}"
```

---

### QUIC/HTTP3 Filtering

QUIC filtering validates QUIC protocol traffic.

#### QUIC Version Filtering

```yaml
apiVersion: pistonprotection.io/v1
kind: FilterRule
metadata:
  name: quic-version-filter
spec:
  match:
    l7Protocols: [QUIC, HTTP3]
    l7Match:
      quic:
        # Only allow QUIC v1 and v2
        allowedVersions: [1, 2]
  action: ALLOW
---
apiVersion: pistonprotection.io/v1
kind: FilterRule
metadata:
  name: block-old-quic
spec:
  match:
    l7Protocols: [QUIC]
  action: DROP
  priority: 1000
```

#### QUIC Initial Packet Validation

```yaml
protection:
  protocols:
    quic:
      enabled: true
      validateInitial: true
      # Connection ID length validation
      minCidLength: 8
      maxCidLength: 20
      # Require retry token during high load
      requireToken: false
      maxPacketSize: 1350
```

#### QUIC Rate Limiting

```yaml
apiVersion: pistonprotection.io/v1
kind: FilterRule
metadata:
  name: quic-rate-limit
spec:
  match:
    l7Protocols: [QUIC]
  action: RATE_LIMIT
  rateLimit:
    requestsPerSecond: 500
    burstSize: 1000
```

---

### Minecraft Java Edition

Minecraft Java Edition uses TCP with a custom protocol. PistonProtection provides deep protocol inspection.

#### Protocol Version Filtering

```yaml
protection:
  protocols:
    minecraft:
      enabled: true
      # Minecraft 1.8 (protocol 47) to 1.21.x (protocol 769)
      minVersion: 47
      maxVersion: 769
```

#### Handshake Validation

```yaml
apiVersion: pistonprotection.io/v1
kind: FilterRule
metadata:
  name: minecraft-handshake-validation
spec:
  backendRef:
    name: my-minecraft-server
  match:
    l7Protocols: [MINECRAFT_JAVA]
    l7Match:
      minecraftJava:
        validateHandshake: true
        # Maximum hostname length in handshake
        maxHostnameLength: 255
        # Maximum packet size
        maxPacketSize: 32767
  action: ALLOW
```

**Handshake validation checks:**
- Valid VarInt encoding
- Protocol version within range
- Server address format
- Next state is valid (1=status, 2=login)

#### Status Ping Protection

Protect against status ping floods:

```yaml
apiVersion: pistonprotection.io/v1
kind: FilterRule
metadata:
  name: minecraft-status-protection
spec:
  match:
    l7Protocols: [MINECRAFT_JAVA]
    l7Match:
      minecraftJava:
        packetTypes: [STATUS_REQUEST, STATUS_PING]
        rateLimitStatus: true
  action: RATE_LIMIT
  rateLimit:
    requestsPerSecond: 5
    burstSize: 10
```

#### Connection Limits

```yaml
apiVersion: pistonprotection.io/v1
kind: FilterRule
metadata:
  name: minecraft-connection-limit
spec:
  match:
    l7Protocols: [MINECRAFT_JAVA]
    l7Match:
      minecraftJava:
        maxConnectionsPerIp: 3
  action: DROP
  # This rule triggers when limit exceeded
```

#### Player Name Validation

```yaml
apiVersion: pistonprotection.io/v1
kind: FilterRule
metadata:
  name: minecraft-player-validation
spec:
  match:
    l7Protocols: [MINECRAFT_JAVA]
    l7Match:
      minecraftJava:
        validatePlayerNames: true
        packetTypes: [LOGIN_START]
  action: ALLOW
```

**Player name validation:**
- 3-16 characters
- Alphanumeric and underscores only
- No duplicate login attempts

#### Complete Minecraft Protection

```yaml
apiVersion: pistonprotection.io/v1
kind: Backend
metadata:
  name: minecraft-server
spec:
  type: MINECRAFT_JAVA
  origins:
    - name: primary
      address: 192.168.1.100
      port: 25565
  protection:
    enabled: true
    level: MEDIUM
    perIpRateLimit:
      requestsPerSecond: 100
      burstSize: 200
    geoIp:
      mode: BLOCK_LIST
      countries: [CN, RU, KP]
    l7Settings:
      minecraft:
        validateHandshake: true
        protectStatus: true
        statusRateLimit: 10
        maxPlayersPerIp: 3
        maxConnectionsPerSecond: 20
        protectQuery: true
        challengeMotd: "Verifying connection..."
```

---

### Minecraft Bedrock Edition

Minecraft Bedrock Edition uses UDP with the RakNet protocol.

#### RakNet Validation

```yaml
protection:
  protocols:
    minecraftBedrock:
      enabled: true
      # Validate RakNet magic bytes
      validateRaknetMagic: true
      maxPacketSize: 1400
```

#### Bedrock Rate Limiting

```yaml
apiVersion: pistonprotection.io/v1
kind: FilterRule
metadata:
  name: bedrock-rate-limit
spec:
  match:
    l7Protocols: [MINECRAFT_BEDROCK]
    l7Match:
      minecraftBedrock:
        packetTypes: [UNCONNECTED_PING]
        rateLimitMotd: true
  action: RATE_LIMIT
  rateLimit:
    requestsPerSecond: 5
    burstSize: 10
```

#### Connection Request Validation

```yaml
apiVersion: pistonprotection.io/v1
kind: FilterRule
metadata:
  name: bedrock-connection-validation
spec:
  match:
    l7Protocols: [MINECRAFT_BEDROCK]
    l7Match:
      minecraftBedrock:
        packetTypes:
          - OPEN_CONNECTION_REQUEST_1
          - OPEN_CONNECTION_REQUEST_2
        validateRaknetMagic: true
        maxConnectionsPerIp: 3
  action: ALLOW
```

---

## GeoIP Blocking

Block or allow traffic based on geographic location.

### Block List Mode

Block specific countries:

```yaml
apiVersion: pistonprotection.io/v1
kind: Backend
metadata:
  name: my-backend
spec:
  protection:
    geoIp:
      mode: BLOCK_LIST
      countries:
        - CN  # China
        - RU  # Russia
        - KP  # North Korea
        - IR  # Iran
```

### Allow List Mode

Only allow specific countries:

```yaml
apiVersion: pistonprotection.io/v1
kind: Backend
metadata:
  name: us-only-backend
spec:
  protection:
    geoIp:
      mode: ALLOW_LIST
      countries:
        - US
        - CA
        - GB
```

### GeoIP Filter Rules

Create granular GeoIP rules:

```yaml
apiVersion: pistonprotection.io/v1
kind: FilterRule
metadata:
  name: challenge-high-risk-countries
spec:
  match:
    sourceCountries:
      - CN
      - RU
      - VN
  action: CHALLENGE
  priority: 100
---
apiVersion: pistonprotection.io/v1
kind: FilterRule
metadata:
  name: block-during-attack
spec:
  match:
    sourceCountryBlacklist:
      - US
      - CA
      - GB
      - DE
      - FR
  action: DROP
  enabled: false  # Enable during attacks
  priority: 50
```

### ASN Filtering

Filter by Autonomous System Number:

```yaml
apiVersion: pistonprotection.io/v1
kind: FilterRule
metadata:
  name: block-known-bad-asns
spec:
  match:
    sourceAsns:
      - AS12345
      - AS67890
  action: DROP
```

### GeoIP Database Updates

```yaml
protection:
  geoip:
    enabled: true
    # Database update interval
    updateInterval: 24h
    # Database provider: maxmind, ip2location, dbip
    provider: maxmind
    # License key (if required)
    licenseKey: ""
```

---

## Rate Limiting

### Per-IP Rate Limiting

```yaml
protection:
  perIpRateLimit:
    requestsPerSecond: 1000
    burstSize: 2000
```

### Global Rate Limiting

```yaml
protection:
  globalRateLimit:
    requestsPerSecond: 1000000
    burstSize: 2000000
```

### Token Bucket Algorithm

PistonProtection uses token bucket rate limiting:

```
┌─────────────────────────────────────────────┐
│                Token Bucket                  │
│                                             │
│  Bucket Size: 2000 tokens                   │
│  Refill Rate: 1000 tokens/second            │
│                                             │
│  ┌─────────────────────────────────────┐   │
│  │ ████████████████░░░░░░░░░░░░░░░░░░░ │   │
│  │ Current: 1500 tokens                │   │
│  └─────────────────────────────────────┘   │
│                                             │
│  Each packet consumes 1 token               │
│  Empty bucket = DROP                        │
└─────────────────────────────────────────────┘
```

### Sliding Window Rate Limiting

For more accurate rate limiting:

```yaml
apiVersion: pistonprotection.io/v1
kind: FilterRule
metadata:
  name: sliding-window-limit
spec:
  match:
    protocols: [TCP, UDP]
  action: RATE_LIMIT
  rateLimit:
    requestsPerSecond: 100
    burstSize: 150
    windowSeconds: 60
    # Use sliding window instead of fixed window
    slidingWindow: true
```

### Connection Rate Limiting

Limit new connections per IP:

```yaml
protection:
  connectionRateLimit:
    connectionsPerSecond: 10
    burstSize: 20
```

---

## Custom Filter Rules

### Rule Structure

```yaml
apiVersion: pistonprotection.io/v1
kind: FilterRule
metadata:
  name: rule-name
  namespace: default
spec:
  # Target backend (optional - global if not specified)
  backendRef:
    name: my-backend

  # Rule priority (lower = higher priority)
  priority: 100

  # Enable/disable rule
  enabled: true

  # Match conditions
  match:
    # Source IP matching
    sourceIps:
      - address: 192.168.1.0
        prefixLength: 24
    sourceIpBlacklist: []

    # GeoIP matching
    sourceCountries: []
    sourceCountryBlacklist: []

    # ASN matching
    sourceAsns: []

    # Destination matching
    destinationIps: []
    destinationPorts: []

    # Protocol matching
    protocols: []
    l7Protocols: []
    l7Match: {}

    # Time-based matching
    timeMatch: {}

  # Action to take
  action: DROP

  # Rate limit config (if action is RATE_LIMIT)
  rateLimit: {}

  # Description
  description: "Rule description"
```

### Rule Priority

Rules are evaluated in priority order (lower number = higher priority):

```
Priority 1   ─────►  Whitelist rules
Priority 100 ─────►  Block known bad IPs
Priority 200 ─────►  GeoIP rules
Priority 500 ─────►  Protocol validation
Priority 1000 ────►  Default rules
```

### Time-Based Rules

Create rules that apply during specific times:

```yaml
apiVersion: pistonprotection.io/v1
kind: FilterRule
metadata:
  name: weekend-strict-mode
spec:
  match:
    timeMatch:
      # Weekend only (0=Sunday, 6=Saturday)
      daysOfWeek: [0, 6]
      # All day (minutes from midnight UTC)
      timeRanges:
        - startMinutes: 0
          endMinutes: 1440
  action: RATE_LIMIT
  rateLimit:
    requestsPerSecond: 500
```

---

## Filter Rule Examples

### Example 1: Comprehensive Whitelist

```yaml
apiVersion: pistonprotection.io/v1
kind: FilterRule
metadata:
  name: whitelist-internal
spec:
  priority: 1
  match:
    sourceIps:
      - address: 10.0.0.0
        prefixLength: 8
      - address: 172.16.0.0
        prefixLength: 12
      - address: 192.168.0.0
        prefixLength: 16
  action: ALLOW
  description: "Allow internal networks"
```

### Example 2: Block Known Attack Sources

```yaml
apiVersion: pistonprotection.io/v1
kind: FilterRule
metadata:
  name: block-threat-intel
spec:
  priority: 50
  match:
    sourceIps:
      # Known botnet C&C
      - address: 185.220.101.0
        prefixLength: 24
      - address: 45.155.205.0
        prefixLength: 24
      - address: 5.188.206.0
        prefixLength: 24
  action: DROP
  description: "Block known malicious IP ranges"
```

### Example 3: DDoS Emergency Rule

```yaml
apiVersion: pistonprotection.io/v1
kind: FilterRule
metadata:
  name: emergency-mode
spec:
  priority: 10
  enabled: false  # Enable during attacks
  match:
    sourceCountryBlacklist:
      - US  # Allow only US traffic
  action: DROP
  description: "Emergency rule - enable during attacks"
```

### Example 4: HTTP Slowloris Protection

```yaml
apiVersion: pistonprotection.io/v1
kind: FilterRule
metadata:
  name: slowloris-protection
spec:
  match:
    l7Protocols: [HTTP, HTTP2]
    l7Match:
      http:
        # Require headers to be sent within timeout
        headerTimeout: 10s
        # Require complete request within timeout
        requestTimeout: 30s
  action: DROP
  description: "Drop slow/incomplete HTTP requests"
```

### Example 5: Gaming Server Protection

```yaml
apiVersion: pistonprotection.io/v1
kind: FilterRule
metadata:
  name: gaming-protection
spec:
  backendRef:
    name: game-server
  priority: 100
  match:
    protocols: [UDP]
    destinationPorts:
      - start: 27015
        end: 27030
  action: RATE_LIMIT
  rateLimit:
    requestsPerSecond: 500
    burstSize: 1000
  description: "Rate limit game server traffic"
```

### Example 6: API Protection

```yaml
apiVersion: pistonprotection.io/v1
kind: FilterRule
metadata:
  name: api-protection
spec:
  backendRef:
    name: api-backend
  match:
    l7Protocols: [HTTP, HTTP2]
    l7Match:
      http:
        paths:
          - "/api/*"
        methods: [GET, POST, PUT, DELETE]
  action: RATE_LIMIT
  rateLimit:
    requestsPerSecond: 100
    burstSize: 200
    # Per-IP rate limiting
    scope: PER_IP
  description: "Rate limit API endpoints"
```

---

## Related Documentation

- [Installation Guide](installation.md) - Initial setup
- [Configuration Reference](configuration.md) - Full configuration options
- [API Documentation](api.md) - REST/gRPC API reference
- [Operations Guide](operations.md) - Monitoring and maintenance
