# Architecture Documentation

This document provides a comprehensive overview of PistonProtection's architecture, including system design, component descriptions, data flows, and Kubernetes integration.

## Table of Contents

- [System Overview](#system-overview)
- [Architecture Principles](#architecture-principles)
- [Component Architecture](#component-architecture)
- [Data Flow](#data-flow)
- [eBPF/XDP Pipeline](#ebpfxdp-pipeline)
- [Kubernetes Integration](#kubernetes-integration)
- [Database Schema](#database-schema)
- [Security Architecture](#security-architecture)
- [Scalability](#scalability)

---

## System Overview

PistonProtection is an enterprise-grade DDoS protection platform built on modern cloud-native technologies. It provides line-rate packet filtering using eBPF/XDP technology at the network driver level.

### High-Level Architecture

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                              Internet Traffic                                  │
└──────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌──────────────────────────────────────────────────────────────────────────────┐
│                          Anycast Edge Network                                 │
│                                                                               │
│   ┌─────────────┐    ┌─────────────┐    ┌─────────────┐                      │
│   │   Worker    │    │   Worker    │    │   Worker    │     ...              │
│   │   Node 1    │    │   Node 2    │    │   Node 3    │                      │
│   │ ┌─────────┐ │    │ ┌─────────┐ │    │ ┌─────────┐ │                      │
│   │ │XDP/eBPF │ │    │ │XDP/eBPF │ │    │ │XDP/eBPF │ │                      │
│   │ │ Filter  │ │    │ │ Filter  │ │    │ │ Filter  │ │                      │
│   │ └─────────┘ │    │ └─────────┘ │    │ └─────────┘ │                      │
│   └─────────────┘    └─────────────┘    └─────────────┘                      │
│          │                  │                  │                              │
│          └──────────────────┼──────────────────┘                              │
│                             │                                                 │
│                             ▼                                                 │
│   ┌─────────────────────────────────────────────────────────────────────┐    │
│   │                    Clean Traffic Output                              │    │
│   └─────────────────────────────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌──────────────────────────────────────────────────────────────────────────────┐
│                       Control Plane (Kubernetes)                              │
│                                                                               │
│   ┌────────────────────────────────────────────────────────────────────┐     │
│   │                        Core Services                                │     │
│   │                                                                     │     │
│   │  ┌────────────┐  ┌────────────┐  ┌────────────┐  ┌────────────┐   │     │
│   │  │  Gateway   │  │  Config    │  │  Metrics   │  │   Auth     │   │     │
│   │  │  Service   │  │  Manager   │  │ Collector  │  │  Service   │   │     │
│   │  └────────────┘  └────────────┘  └────────────┘  └────────────┘   │     │
│   │                                                                     │     │
│   │  ┌────────────┐                                                    │     │
│   │  │ Kubernetes │                                                    │     │
│   │  │  Operator  │                                                    │     │
│   │  └────────────┘                                                    │     │
│   └────────────────────────────────────────────────────────────────────┘     │
│                                                                               │
│   ┌────────────────────────────────────────────────────────────────────┐     │
│   │                        Data Stores                                  │     │
│   │                                                                     │     │
│   │  ┌────────────┐  ┌────────────┐  ┌────────────┐                   │     │
│   │  │ PostgreSQL │  │   Redis    │  │ Prometheus │                   │     │
│   │  └────────────┘  └────────────┘  └────────────┘                   │     │
│   └────────────────────────────────────────────────────────────────────┘     │
│                                                                               │
│   ┌────────────────────────────────────────────────────────────────────┐     │
│   │                        Observability                                │     │
│   │                                                                     │     │
│   │  ┌────────────┐  ┌────────────┐  ┌────────────┐                   │     │
│   │  │  Grafana   │  │    Loki    │  │   Hubble   │                   │     │
│   │  └────────────┘  └────────────┘  └────────────┘                   │     │
│   └────────────────────────────────────────────────────────────────────┘     │
│                                                                               │
│   ┌────────────────────────────────────────────────────────────────────┐     │
│   │                        Dashboard                                    │     │
│   │                                                                     │     │
│   │  ┌────────────────────────────────────────────────────────────┐   │     │
│   │  │              Frontend (TanStack Start)                      │   │     │
│   │  └────────────────────────────────────────────────────────────┘   │     │
│   └────────────────────────────────────────────────────────────────────┘     │
└──────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌──────────────────────────────────────────────────────────────────────────────┐
│                          Origin Servers                                       │
│                                                                               │
│   ┌────────────┐    ┌────────────┐    ┌────────────┐                         │
│   │   Origin   │    │   Origin   │    │   Origin   │     ...                 │
│   │  Server 1  │    │  Server 2  │    │  Server 3  │                         │
│   └────────────┘    └────────────┘    └────────────┘                         │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## Architecture Principles

### Design Goals

1. **Line-Rate Filtering**: Process packets at NIC speed using XDP
2. **Zero Trust**: All traffic is untrusted by default
3. **Cloud Native**: Kubernetes-first deployment model
4. **Horizontal Scalability**: Scale workers based on traffic
5. **Multi-Tenancy**: Support multiple organizations
6. **Observability**: Deep visibility into traffic and attacks

### Technology Choices

| Component | Technology | Rationale |
|-----------|------------|-----------|
| Packet Filtering | eBPF/XDP | Line-rate performance, kernel bypass |
| Backend Services | Rust | Memory safety, performance |
| Frontend | TanStack Start | Modern React, SSR support |
| CNI | Cilium | eBPF-native networking |
| Database | PostgreSQL | ACID compliance, JSON support |
| Cache | Redis | Low-latency state sharing |
| Orchestration | Kubernetes | Industry standard, extensible |

---

## Component Architecture

### Gateway Service

The Gateway service is the API layer for PistonProtection.

```
┌─────────────────────────────────────────────────────────────┐
│                     Gateway Service                          │
│                                                              │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐            │
│  │    HTTP    │  │    gRPC    │  │  WebSocket │            │
│  │  Handler   │  │  Handler   │  │  Handler   │            │
│  └─────┬──────┘  └─────┬──────┘  └─────┬──────┘            │
│        │               │               │                    │
│        └───────────────┼───────────────┘                    │
│                        │                                    │
│  ┌─────────────────────┴─────────────────────┐             │
│  │              Middleware Stack              │             │
│  │  ┌──────────────────────────────────────┐ │             │
│  │  │  Auth → RateLimit → Logging → Trace  │ │             │
│  │  └──────────────────────────────────────┘ │             │
│  └─────────────────────┬─────────────────────┘             │
│                        │                                    │
│  ┌─────────────────────┴─────────────────────┐             │
│  │              Service Layer                 │             │
│  │                                            │             │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐  │             │
│  │  │ Backend  │ │  Filter  │ │ Metrics  │  │             │
│  │  │ Service  │ │ Service  │ │ Service  │  │             │
│  │  └──────────┘ └──────────┘ └──────────┘  │             │
│  └───────────────────────────────────────────┘             │
│                        │                                    │
│  ┌─────────────────────┴─────────────────────┐             │
│  │              Data Layer                    │             │
│  │                                            │             │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐  │             │
│  │  │PostgreSQL│ │  Redis   │ │  gRPC    │  │             │
│  │  │  Client  │ │  Client  │ │ Clients  │  │             │
│  │  └──────────┘ └──────────┘ └──────────┘  │             │
│  └───────────────────────────────────────────┘             │
└─────────────────────────────────────────────────────────────┘
```

**Responsibilities:**
- REST API for dashboard and integrations
- gRPC API for high-performance clients
- WebSocket for real-time metrics
- Authentication and authorization
- Request validation and rate limiting

### Worker Service

The Worker service performs packet filtering using eBPF/XDP.

```
┌─────────────────────────────────────────────────────────────┐
│                      Worker Service                          │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │                  User Space                           │   │
│  │                                                       │   │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐           │   │
│  │  │  Config  │  │  Metrics │  │  Health  │           │   │
│  │  │ Receiver │  │ Reporter │  │ Reporter │           │   │
│  │  └────┬─────┘  └────┬─────┘  └──────────┘           │   │
│  │       │              │                               │   │
│  │  ┌────┴──────────────┴────┐                         │   │
│  │  │     eBPF Map Manager   │                         │   │
│  │  └────────────┬───────────┘                         │   │
│  │               │                                      │   │
│  └───────────────┼──────────────────────────────────────┘   │
│                  │                                          │
│  ┌───────────────┴──────────────────────────────────────┐   │
│  │                  Kernel Space                         │   │
│  │                                                       │   │
│  │  ┌────────────────────────────────────────────────┐  │   │
│  │  │              XDP Program                        │  │   │
│  │  │                                                 │  │   │
│  │  │  ┌─────────┐ ┌─────────┐ ┌─────────┐          │  │   │
│  │  │  │  Parse  │→│  Match  │→│ Action  │          │  │   │
│  │  │  │ Headers │ │  Rules  │ │ Execute │          │  │   │
│  │  │  └─────────┘ └─────────┘ └─────────┘          │  │   │
│  │  └────────────────────────────────────────────────┘  │   │
│  │                                                       │   │
│  │  ┌─────────────────────────────────────────────────┐ │   │
│  │  │              eBPF Maps                           │ │   │
│  │  │                                                  │ │   │
│  │  │  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐  │ │   │
│  │  │  │Config  │ │ Blocked│ │  Rate  │ │ConnTrack│  │ │   │
│  │  │  │  Map   │ │IP Map  │ │Limit   │ │  Map   │  │ │   │
│  │  │  └────────┘ └────────┘ └────────┘ └────────┘  │ │   │
│  │  └─────────────────────────────────────────────────┘ │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │                Network Interface                      │   │
│  │                     (eth0)                            │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

**Responsibilities:**
- Attach XDP programs to network interfaces
- Receive configuration from control plane
- Update eBPF maps with rules
- Report metrics to control plane
- Connection tracking state management

### Config Manager

Distributes configuration to workers.

```
┌─────────────────────────────────────────────────────────────┐
│                   Config Manager                             │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │                 Configuration Store                   │   │
│  │                                                       │   │
│  │  ┌──────────┐     ┌──────────┐     ┌──────────┐     │   │
│  │  │PostgreSQL│ ←→  │  Cache   │ ←→  │ Watcher  │     │   │
│  │  │ Backend  │     │ (Redis)  │     │          │     │   │
│  │  └──────────┘     └──────────┘     └──────────┘     │   │
│  └──────────────────────────────────────────────────────┘   │
│                           │                                  │
│  ┌────────────────────────┴─────────────────────────────┐   │
│  │                Configuration Distributor              │   │
│  │                                                       │   │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐              │   │
│  │  │ Worker  │  │ Worker  │  │ Worker  │              │   │
│  │  │ Client  │  │ Client  │  │ Client  │              │   │
│  │  └─────────┘  └─────────┘  └─────────┘              │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

**Responsibilities:**
- Store configuration in PostgreSQL
- Cache frequently accessed config in Redis
- Push config updates to workers via gRPC streams
- Version configuration for atomic updates
- Handle configuration validation

### Metrics Service

Aggregates and stores metrics from all components.

```
┌─────────────────────────────────────────────────────────────┐
│                    Metrics Service                           │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │                  Metrics Ingestion                    │   │
│  │                                                       │   │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐              │   │
│  │  │ Worker  │  │ Gateway │  │Operator │              │   │
│  │  │ Metrics │  │ Metrics │  │ Metrics │              │   │
│  │  └────┬────┘  └────┬────┘  └────┬────┘              │   │
│  │       └────────────┼────────────┘                    │   │
│  │                    │                                 │   │
│  │  ┌─────────────────┴──────────────────┐             │   │
│  │  │           Aggregator               │             │   │
│  │  └─────────────────┬──────────────────┘             │   │
│  └────────────────────┼─────────────────────────────────┘   │
│                       │                                      │
│  ┌────────────────────┴─────────────────────────────────┐   │
│  │                  Storage Layer                        │   │
│  │                                                       │   │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐           │   │
│  │  │   Raw    │  │Aggregated│  │  Alert   │           │   │
│  │  │ Metrics  │  │ Metrics  │  │  Rules   │           │   │
│  │  └──────────┘  └──────────┘  └──────────┘           │   │
│  └──────────────────────────────────────────────────────┘   │
│                       │                                      │
│  ┌────────────────────┴─────────────────────────────────┐   │
│  │                   Export Layer                        │   │
│  │                                                       │   │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐           │   │
│  │  │Prometheus│  │  Query   │  │  Alert   │           │   │
│  │  │ Exporter │  │   API    │  │ Manager  │           │   │
│  │  └──────────┘  └──────────┘  └──────────┘           │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

**Responsibilities:**
- Collect metrics from all components
- Aggregate metrics at multiple time intervals
- Store time-series data
- Provide query API for dashboard
- Evaluate alert rules and send notifications

### Kubernetes Operator

Manages custom resources for PistonProtection.

```
┌─────────────────────────────────────────────────────────────┐
│                  Kubernetes Operator                         │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │                  Controller Manager                   │   │
│  │                                                       │   │
│  │  ┌────────────────────────────────────────────────┐  │   │
│  │  │              Reconcilers                        │  │   │
│  │  │                                                 │  │   │
│  │  │  ┌──────────┐ ┌──────────┐ ┌──────────────┐   │  │   │
│  │  │  │ Backend  │ │  Filter  │ │ DDoSProtection│   │  │   │
│  │  │  │Controller│ │Controller│ │  Controller  │   │  │   │
│  │  │  └──────────┘ └──────────┘ └──────────────┘   │  │   │
│  │  └────────────────────────────────────────────────┘  │   │
│  │                                                       │   │
│  │  ┌────────────────────────────────────────────────┐  │   │
│  │  │              CRD Definitions                    │  │   │
│  │  │                                                 │  │   │
│  │  │  • backends.pistonprotection.io                │  │   │
│  │  │  • filterrules.pistonprotection.io             │  │   │
│  │  │  • ddosprotections.pistonprotection.io         │  │   │
│  │  └────────────────────────────────────────────────┘  │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │                  Kubernetes API                       │   │
│  │                                                       │   │
│  │  Watch CRDs  →  Reconcile  →  Update Status          │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

**Responsibilities:**
- Manage Backend CRDs
- Manage FilterRule CRDs
- Manage DDoSProtection CRDs
- Reconcile desired state with actual state
- Update resource status

---

## Data Flow

### Packet Processing Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ 1. Packet Arrives at NIC                                                     │
│    └─→ XDP hook triggered before kernel stack                               │
└─────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ 2. XDP: Parse Ethernet Header                                                │
│    └─→ Extract: src/dst MAC, EtherType                                      │
│    └─→ Drop non-IP traffic                                                  │
└─────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ 3. XDP: Parse IP Header                                                      │
│    └─→ Extract: src/dst IP, protocol, TTL                                   │
│    └─→ Check blocked IP map                                                 │
│    └─→ Check GeoIP map                                                      │
└─────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ 4. XDP: Rate Limiting                                                        │
│    └─→ Look up rate limit entry for source IP                               │
│    └─→ Token bucket: consume token or drop                                  │
│    └─→ Update rate limit map                                                │
└─────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ 5. XDP: Protocol Validation (TCP/UDP)                                        │
│    └─→ Parse L4 header                                                      │
│    └─→ TCP: SYN cookie validation                                           │
│    └─→ UDP: Amplification check                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ 6. XDP: L7 Protocol Detection                                                │
│    └─→ Minecraft: Validate handshake                                        │
│    └─→ QUIC: Validate initial packet                                        │
│    └─→ HTTP: Basic header validation                                        │
└─────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ 7. XDP: Custom Rules                                                         │
│    └─→ Match against filter rules                                           │
│    └─→ Execute action: PASS, DROP, REDIRECT                                 │
└─────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ 8. XDP: Action                                                               │
│    └─→ XDP_PASS: Continue to kernel stack                                   │
│    └─→ XDP_DROP: Drop packet                                                │
│    └─→ XDP_REDIRECT: Send to different interface                            │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Configuration Update Flow

```
┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐
│Dashboard │     │ Gateway  │     │  Config  │     │  Worker  │
│          │     │          │     │  Manager │     │          │
└────┬─────┘     └────┬─────┘     └────┬─────┘     └────┬─────┘
     │                │                │                │
     │ Update Rule    │                │                │
     │───────────────>│                │                │
     │                │                │                │
     │                │ Save to DB     │                │
     │                │───────────────>│                │
     │                │                │                │
     │                │                │ Broadcast      │
     │                │                │ via gRPC       │
     │                │                │───────────────>│
     │                │                │                │
     │                │                │                │ Update
     │                │                │                │ eBPF Map
     │                │                │                │───┐
     │                │                │                │   │
     │                │                │                │<──┘
     │                │                │                │
     │                │                │  Acknowledge   │
     │                │                │<───────────────│
     │                │                │                │
     │                │  Confirm       │                │
     │                │<───────────────│                │
     │                │                │                │
     │  Success       │                │                │
     │<───────────────│                │                │
     │                │                │                │
```

### Metrics Flow

```
┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐
│  Worker  │     │ Metrics  │     │Prometheus│     │ Grafana  │
│  (XDP)   │     │ Service  │     │          │     │          │
└────┬─────┘     └────┬─────┘     └────┬─────┘     └────┬─────┘
     │                │                │                │
     │ Per-CPU        │                │                │
     │ Counters       │                │                │
     │───────────────>│                │                │
     │ (Every 1s)     │                │                │
     │                │                │                │
     │                │ Aggregate &    │                │
     │                │ Store          │                │
     │                │───┐            │                │
     │                │   │            │                │
     │                │<──┘            │                │
     │                │                │                │
     │                │                │ Scrape         │
     │                │                │ /metrics       │
     │                │<───────────────│                │
     │                │                │                │
     │                │                │                │ Query
     │                │                │                │ PromQL
     │                │                │<───────────────│
     │                │                │                │
     │                │                │ Results        │
     │                │                │───────────────>│
     │                │                │                │
```

---

## eBPF/XDP Pipeline

### XDP Program Structure

```c
// Simplified XDP program flow
SEC("xdp")
int xdp_filter(struct xdp_md *ctx) {
    // 1. Parse Ethernet header
    struct ethhdr *eth = parse_eth(ctx);
    if (!eth || eth->h_proto != htons(ETH_P_IP))
        return XDP_PASS;

    // 2. Parse IP header
    struct iphdr *ip = parse_ip(ctx);
    if (!ip)
        return XDP_DROP;

    // 3. Check blocked IP map
    if (is_blocked(ip->saddr))
        return XDP_DROP;

    // 4. Check GeoIP
    if (!is_allowed_country(ip->saddr))
        return XDP_DROP;

    // 5. Rate limiting
    if (!check_rate_limit(ip->saddr))
        return XDP_DROP;

    // 6. Protocol-specific validation
    if (ip->protocol == IPPROTO_TCP) {
        return handle_tcp(ctx, ip);
    } else if (ip->protocol == IPPROTO_UDP) {
        return handle_udp(ctx, ip);
    }

    // 7. Custom rules
    return apply_custom_rules(ctx, ip);
}
```

### eBPF Maps

| Map | Type | Purpose |
|-----|------|---------|
| `CONFIG_MAP` | Array | Global configuration |
| `BLOCKED_IPS` | LPM Trie | Blocked IP/CIDR lookup |
| `ALLOWED_IPS` | LPM Trie | Whitelisted IPs |
| `RATE_LIMIT_MAP` | Per-CPU Hash | Rate limiting state |
| `CONN_TRACK_MAP` | Hash | Connection tracking |
| `GEOIP_MAP` | Hash | IP to country mapping |
| `FILTER_RULES` | Array | Custom filter rules |
| `METRICS_MAP` | Per-CPU Array | Per-CPU counters |

### XDP Attachment Modes

```
┌─────────────────────────────────────────────────────────────┐
│                    XDP Attachment Modes                      │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  Native Mode (XDP_FLAGS_DRV_MODE)                           │
│  ├─ Executes in NIC driver                                  │
│  ├─ Best performance                                        │
│  └─ Requires driver support                                 │
│                                                              │
│  Generic Mode (XDP_FLAGS_SKB_MODE)                          │
│  ├─ Executes in network stack                               │
│  ├─ Works with any driver                                   │
│  └─ Lower performance                                       │
│                                                              │
│  Offload Mode (XDP_FLAGS_HW_MODE)                           │
│  ├─ Executes on NIC hardware                                │
│  ├─ Highest performance                                     │
│  └─ Limited NIC support                                     │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## Kubernetes Integration

### Custom Resource Definitions

```yaml
# Backend CRD
apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition
metadata:
  name: backends.pistonprotection.io
spec:
  group: pistonprotection.io
  names:
    kind: Backend
    plural: backends
    singular: backend
    shortNames: [bk]
  scope: Namespaced
  versions:
    - name: v1
      served: true
      storage: true
      schema:
        openAPIV3Schema:
          type: object
          properties:
            spec:
              type: object
              properties:
                type:
                  type: string
                  enum: [HTTP, HTTPS, TCP, UDP, MINECRAFT_JAVA, MINECRAFT_BEDROCK, QUIC]
                origins:
                  type: array
                  items:
                    type: object
                protection:
                  type: object
            status:
              type: object
              properties:
                health:
                  type: string
                conditions:
                  type: array
```

### RBAC Configuration

```yaml
# Operator service account
apiVersion: v1
kind: ServiceAccount
metadata:
  name: pistonprotection-operator
  namespace: pistonprotection

---
# Cluster role for operator
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: pistonprotection-operator
rules:
  - apiGroups: ["pistonprotection.io"]
    resources: ["*"]
    verbs: ["*"]
  - apiGroups: [""]
    resources: ["pods", "services", "configmaps", "secrets"]
    verbs: ["get", "list", "watch"]
  - apiGroups: ["apps"]
    resources: ["deployments", "daemonsets"]
    verbs: ["get", "list", "watch", "update"]
```

### Network Policy

```yaml
# Worker network policy
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: pistonprotection-worker
  namespace: pistonprotection
spec:
  podSelector:
    matchLabels:
      app.kubernetes.io/component: worker
  policyTypes:
    - Ingress
    - Egress
  ingress:
    - from:
        - podSelector:
            matchLabels:
              app.kubernetes.io/name: pistonprotection
  egress:
    - to:
        - podSelector:
            matchLabels:
              app.kubernetes.io/component: config-mgr
    - to:
        - podSelector:
            matchLabels:
              app.kubernetes.io/component: metrics
```

---

## Database Schema

### Core Tables

```sql
-- Organizations
CREATE TABLE organizations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Backends
CREATE TABLE backends (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id),
    name VARCHAR(255) NOT NULL,
    type VARCHAR(50) NOT NULL,
    config JSONB NOT NULL DEFAULT '{}',
    protection JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Filter Rules
CREATE TABLE filter_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    backend_id UUID REFERENCES backends(id),
    name VARCHAR(255) NOT NULL,
    priority INTEGER NOT NULL DEFAULT 1000,
    enabled BOOLEAN NOT NULL DEFAULT true,
    match_config JSONB NOT NULL,
    action VARCHAR(50) NOT NULL,
    rate_limit JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Metrics (time-series)
CREATE TABLE metrics (
    time TIMESTAMPTZ NOT NULL,
    backend_id UUID NOT NULL,
    metric_name VARCHAR(100) NOT NULL,
    value DOUBLE PRECISION NOT NULL
);

-- Convert to hypertable (TimescaleDB)
SELECT create_hypertable('metrics', 'time');
```

---

## Security Architecture

### Authentication Flow

```
┌────────────┐     ┌────────────┐     ┌────────────┐
│   Client   │     │  Gateway   │     │Auth Service│
└─────┬──────┘     └─────┬──────┘     └─────┬──────┘
      │                  │                  │
      │ Login Request    │                  │
      │─────────────────>│                  │
      │                  │                  │
      │                  │ Validate         │
      │                  │─────────────────>│
      │                  │                  │
      │                  │   JWT Token      │
      │                  │<─────────────────│
      │                  │                  │
      │   JWT Token      │                  │
      │<─────────────────│                  │
      │                  │                  │
      │ API Request      │                  │
      │ + JWT Token      │                  │
      │─────────────────>│                  │
      │                  │                  │
      │                  │ Verify Token     │
      │                  │───┐              │
      │                  │   │              │
      │                  │<──┘              │
      │                  │                  │
      │   Response       │                  │
      │<─────────────────│                  │
```

### Defense in Depth

```
┌─────────────────────────────────────────────────────────────┐
│                    Security Layers                           │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  Layer 1: Network                                           │
│  ├─ XDP packet filtering                                    │
│  ├─ Network policies                                        │
│  └─ TLS encryption                                          │
│                                                              │
│  Layer 2: Authentication                                    │
│  ├─ JWT tokens                                              │
│  ├─ API keys                                                │
│  └─ OAuth 2.0                                               │
│                                                              │
│  Layer 3: Authorization                                     │
│  ├─ RBAC                                                    │
│  ├─ Resource-based access                                   │
│  └─ Organization isolation                                  │
│                                                              │
│  Layer 4: Data                                              │
│  ├─ Encryption at rest                                      │
│  ├─ Audit logging                                           │
│  └─ Data isolation                                          │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## Scalability

### Horizontal Scaling

```
┌─────────────────────────────────────────────────────────────┐
│                    Scaling Strategy                          │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  Gateway Service                                            │
│  ├─ HPA based on CPU/Memory                                 │
│  ├─ Min: 2, Max: 20 replicas                                │
│  └─ Target: 70% CPU utilization                             │
│                                                              │
│  Worker Service (DaemonSet)                                 │
│  ├─ Runs on dedicated worker nodes                          │
│  ├─ Scale by adding worker nodes                            │
│  └─ Anycast for traffic distribution                        │
│                                                              │
│  Metrics Service                                            │
│  ├─ HPA based on queue depth                                │
│  ├─ Sharded by backend ID                                   │
│  └─ Async processing                                        │
│                                                              │
│  Database                                                   │
│  ├─ Read replicas for queries                               │
│  ├─ Connection pooling                                      │
│  └─ Partitioning for time-series                            │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### Performance Targets

| Metric | Target |
|--------|--------|
| Packet Processing | 10+ Mpps per worker |
| Latency Overhead | < 1 microsecond |
| API Response Time | < 50ms (p99) |
| Configuration Update | < 100ms propagation |
| Metric Collection | 1 second granularity |

---

## Related Documentation

- [Installation Guide](installation.md) - Deployment instructions
- [Configuration Reference](configuration.md) - All configuration options
- [Development Guide](development.md) - Contributing and development
- [Operations Guide](operations.md) - Production operations
