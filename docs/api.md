# API Documentation

PistonProtection provides comprehensive REST and gRPC APIs for programmatic access to all platform features.

## Table of Contents

- [Authentication](#authentication)
  - [API Keys](#api-keys)
  - [JWT Tokens](#jwt-tokens)
  - [OAuth 2.0](#oauth-20)
- [REST API Reference](#rest-api-reference)
  - [Backends](#backends)
  - [Filter Rules](#filter-rules)
  - [Metrics](#metrics)
  - [IP Management](#ip-management)
  - [Organizations](#organizations)
  - [Users](#users)
  - [Alerts](#alerts)
  - [Audit Logs](#audit-logs)
- [gRPC API Reference](#grpc-api-reference)
- [Rate Limits](#rate-limits)
- [Error Handling](#error-handling)
- [Webhooks](#webhooks)
- [SDKs and Examples](#sdks-and-examples)

---

## Authentication

PistonProtection supports multiple authentication methods.

### API Keys

API keys are the recommended method for programmatic access.

#### Creating an API Key

1. Navigate to **Settings > API Keys** in the dashboard
2. Click **Create API Key**
3. Select permissions and set expiration
4. Copy the key immediately (it won't be shown again)

#### Using API Keys

Include the API key in the `Authorization` header:

```bash
curl -H "Authorization: Bearer pp_live_xxxxxxxxxxxxxxxxxxxx" \
    https://api.pistonprotection.io/v1/backends
```

#### API Key Permissions

| Permission | Description |
|------------|-------------|
| `read` | Read-only access to resources |
| `write` | Create and update resources |
| `delete` | Delete resources |
| `admin` | Full administrative access |

#### API Key Format

- **Live keys**: `pp_live_` prefix (production)
- **Test keys**: `pp_test_` prefix (sandbox)

### JWT Tokens

For user-authenticated sessions, use JWT tokens.

#### Obtaining a Token

```http
POST /v1/auth/login
Content-Type: application/json

{
    "email": "user@example.com",
    "password": "your-password"
}
```

Response:

```json
{
    "accessToken": "eyJhbGciOiJIUzI1NiIs...",
    "refreshToken": "eyJhbGciOiJIUzI1NiIs...",
    "expiresIn": 900,
    "tokenType": "Bearer"
}
```

#### Refreshing Tokens

```http
POST /v1/auth/refresh
Content-Type: application/json

{
    "refreshToken": "eyJhbGciOiJIUzI1NiIs..."
}
```

### OAuth 2.0

PistonProtection supports OAuth 2.0 for third-party integrations.

#### Supported Flows

- Authorization Code (recommended for web apps)
- Client Credentials (for server-to-server)

#### Authorization URL

```
https://api.pistonprotection.io/oauth/authorize?
    response_type=code&
    client_id=YOUR_CLIENT_ID&
    redirect_uri=https://your-app.com/callback&
    scope=read write&
    state=random-state-string
```

#### Token Exchange

```http
POST /oauth/token
Content-Type: application/x-www-form-urlencoded

grant_type=authorization_code&
code=AUTHORIZATION_CODE&
client_id=YOUR_CLIENT_ID&
client_secret=YOUR_CLIENT_SECRET&
redirect_uri=https://your-app.com/callback
```

---

## REST API Reference

**Base URL**: `https://api.pistonprotection.io/v1`

All timestamps are in ISO 8601 format. All responses use JSON.

### Backends

Backends represent protected origin servers.

#### List Backends

```http
GET /v1/backends
```

**Query Parameters**:

| Parameter | Type | Description |
|-----------|------|-------------|
| `page` | integer | Page number (default: 1) |
| `pageSize` | integer | Items per page (default: 20, max: 100) |
| `type` | string | Filter by backend type |
| `status` | string | Filter by status (healthy, degraded, unhealthy) |
| `search` | string | Search by name or domain |

**Response**:

```json
{
    "data": [
        {
            "id": "bk_abc123def456",
            "organizationId": "org_xyz789",
            "name": "mc.example.com",
            "type": "MINECRAFT_JAVA",
            "status": {
                "health": "HEALTHY",
                "healthyOrigins": 2,
                "totalOrigins": 2,
                "requestsPerSecond": 1250,
                "bytesPerSecond": 52428800,
                "underAttack": false
            },
            "origins": [
                {
                    "id": "or_111222333",
                    "name": "primary",
                    "address": "192.168.1.100",
                    "port": 25565,
                    "weight": 100,
                    "priority": 1,
                    "healthStatus": "HEALTHY",
                    "enabled": true
                }
            ],
            "domains": ["mc.example.com", "play.example.com"],
            "protection": {
                "enabled": true,
                "level": "MEDIUM",
                "globalRateLimit": {
                    "requestsPerSecond": 100000,
                    "burstSize": 200000
                },
                "perIpRateLimit": {
                    "requestsPerSecond": 1000,
                    "burstSize": 2000
                }
            },
            "createdAt": "2024-01-15T10:30:00Z",
            "updatedAt": "2024-01-15T14:22:00Z"
        }
    ],
    "pagination": {
        "page": 1,
        "pageSize": 20,
        "totalCount": 5,
        "hasNext": false
    }
}
```

#### Get Backend

```http
GET /v1/backends/{backendId}
```

#### Create Backend

```http
POST /v1/backends
Content-Type: application/json

{
    "name": "mc.example.com",
    "type": "MINECRAFT_JAVA",
    "origins": [
        {
            "name": "primary",
            "address": "192.168.1.100",
            "port": 25565,
            "weight": 100,
            "priority": 1,
            "settings": {
                "connectTimeoutMs": 5000,
                "readTimeoutMs": 30000,
                "writeTimeoutMs": 30000,
                "maxConnections": 1000,
                "proxyProtocol": 2
            }
        },
        {
            "name": "secondary",
            "address": "192.168.1.101",
            "port": 25565,
            "weight": 100,
            "priority": 2
        }
    ],
    "loadBalancer": {
        "algorithm": "ROUND_ROBIN",
        "stickySessions": false
    },
    "healthCheck": {
        "enabled": true,
        "intervalSeconds": 30,
        "timeoutSeconds": 10,
        "healthyThreshold": 2,
        "unhealthyThreshold": 3,
        "minecraft": {
            "queryStatus": true
        }
    },
    "protection": {
        "enabled": true,
        "level": "MEDIUM",
        "perIpRateLimit": {
            "requestsPerSecond": 1000,
            "burstSize": 2000
        },
        "geoIp": {
            "mode": "BLOCK_LIST",
            "countries": ["CN", "RU"]
        },
        "challenge": {
            "enabled": true,
            "type": "JAVASCRIPT",
            "difficulty": 5
        }
    },
    "domains": ["mc.example.com"]
}
```

**Backend Types**:

| Type | Description |
|------|-------------|
| `HTTP` | HTTP/1.1 web traffic |
| `HTTPS` | HTTPS web traffic |
| `TCP` | Generic TCP |
| `UDP` | Generic UDP |
| `MINECRAFT_JAVA` | Minecraft Java Edition |
| `MINECRAFT_BEDROCK` | Minecraft Bedrock Edition |
| `QUIC` | QUIC/HTTP3 |

#### Update Backend

```http
PUT /v1/backends/{backendId}
Content-Type: application/json

{
    "name": "mc.example.com",
    "protection": {
        "level": "HIGH"
    }
}
```

#### Delete Backend

```http
DELETE /v1/backends/{backendId}
```

#### Add Origin to Backend

```http
POST /v1/backends/{backendId}/origins
Content-Type: application/json

{
    "name": "tertiary",
    "address": "192.168.1.102",
    "port": 25565,
    "weight": 50,
    "priority": 3
}
```

#### Update Origin

```http
PUT /v1/backends/{backendId}/origins/{originId}
Content-Type: application/json

{
    "weight": 100,
    "enabled": false
}
```

#### Remove Origin

```http
DELETE /v1/backends/{backendId}/origins/{originId}
```

#### Set Protection Level

Quickly change protection level:

```http
POST /v1/backends/{backendId}/protection/level
Content-Type: application/json

{
    "level": "UNDER_ATTACK"
}
```

**Protection Levels**:

| Level | Description |
|-------|-------------|
| `OFF` | Protection disabled |
| `LOW` | Basic rate limiting |
| `MEDIUM` | Standard protection |
| `HIGH` | Aggressive filtering |
| `UNDER_ATTACK` | Maximum protection |

#### Get Backend Status

```http
GET /v1/backends/{backendId}/status
```

Response:

```json
{
    "health": "HEALTHY",
    "healthyOrigins": 2,
    "totalOrigins": 2,
    "requestsPerSecond": 1250,
    "bytesPerSecond": 52428800,
    "packetsPerSecond": 8500,
    "activeConnections": 2341,
    "underAttack": false,
    "lastUpdated": "2024-01-15T14:22:00Z"
}
```

---

### Filter Rules

Filter rules define traffic filtering logic.

#### List Filter Rules

```http
GET /v1/filters
```

**Query Parameters**:

| Parameter | Type | Description |
|-----------|------|-------------|
| `backendId` | string | Filter by backend |
| `enabled` | boolean | Filter by enabled status |
| `action` | string | Filter by action type |

**Response**:

```json
{
    "data": [
        {
            "id": "fr_abc123",
            "backendId": "bk_xyz789",
            "name": "Block Known Botnets",
            "description": "Blocks IPs from known botnet ranges",
            "priority": 100,
            "enabled": true,
            "match": {
                "sourceIps": [
                    {"address": "185.220.101.0", "prefixLength": 24},
                    {"address": "45.155.205.0", "prefixLength": 24}
                ],
                "sourceCountryBlacklist": ["CN", "RU"]
            },
            "action": "DROP",
            "stats": {
                "packetsMatched": 1234567,
                "bytesMatched": 987654321,
                "packetsDropped": 1234567,
                "lastMatched": "2024-01-15T14:20:00Z"
            },
            "createdAt": "2024-01-10T08:00:00Z",
            "updatedAt": "2024-01-15T12:00:00Z"
        }
    ],
    "pagination": {
        "page": 1,
        "pageSize": 20,
        "totalCount": 12,
        "hasNext": false
    }
}
```

#### Create Filter Rule

```http
POST /v1/filters
Content-Type: application/json

{
    "backendId": "bk_xyz789",
    "name": "Rate Limit Aggressive IPs",
    "description": "Rate limits IPs sending too many packets",
    "priority": 200,
    "enabled": true,
    "match": {
        "protocols": ["TCP"],
        "destinationPorts": [{"start": 25565, "end": 25565}],
        "l7Protocols": ["MINECRAFT_JAVA"],
        "l7Match": {
            "minecraftJava": {
                "validateHandshake": true,
                "maxConnectionsPerIp": 5
            }
        }
    },
    "action": "RATE_LIMIT",
    "rateLimit": {
        "requestsPerSecond": 100,
        "burstSize": 200,
        "windowSeconds": 60
    }
}
```

**Match Conditions**:

```json
{
    "match": {
        "sourceIps": [],
        "sourceIpBlacklist": [],
        "sourceCountries": [],
        "sourceCountryBlacklist": [],
        "sourceAsns": [],
        "destinationIps": [],
        "destinationPorts": [],
        "protocols": ["TCP", "UDP", "ICMP"],
        "l7Protocols": ["HTTP", "MINECRAFT_JAVA", "QUIC"],
        "l7Match": {
            "http": {
                "methods": ["GET", "POST"],
                "paths": ["/api/*"],
                "hosts": ["example.com"],
                "headers": {"User-Agent": "*bot*"},
                "requireTls": true
            },
            "minecraftJava": {
                "minProtocolVersion": 47,
                "maxProtocolVersion": 769,
                "validateHandshake": true,
                "maxConnectionsPerIp": 5
            },
            "minecraftBedrock": {
                "validateRaknetMagic": true,
                "maxConnectionsPerIp": 5
            },
            "quic": {
                "allowedVersions": [1, 2],
                "validateInitial": true
            }
        },
        "timeMatch": {
            "daysOfWeek": [0, 6],
            "timeRanges": [{"startMinutes": 0, "endMinutes": 480}]
        }
    }
}
```

**Actions**:

| Action | Description |
|--------|-------------|
| `ALLOW` | Allow traffic through (whitelist) |
| `DROP` | Silently drop packets |
| `RATE_LIMIT` | Apply rate limiting |
| `CHALLENGE` | Send L7 challenge |
| `LOG` | Log only, no action |
| `REDIRECT` | Redirect to honeypot |

#### Update Filter Rule

```http
PUT /v1/filters/{ruleId}
Content-Type: application/json

{
    "priority": 150,
    "enabled": false
}
```

#### Delete Filter Rule

```http
DELETE /v1/filters/{ruleId}
```

#### Reorder Filter Rules

```http
POST /v1/filters/reorder
Content-Type: application/json

{
    "backendId": "bk_xyz789",
    "ruleIds": ["fr_rule1", "fr_rule2", "fr_rule3"]
}
```

#### Get Rule Statistics

```http
GET /v1/filters/{ruleId}/stats
```

**Query Parameters**:

| Parameter | Type | Description |
|-----------|------|-------------|
| `from` | timestamp | Start time |
| `to` | timestamp | End time |
| `granularity` | string | MINUTE, HOUR, DAY |

---

### Metrics

#### Get Traffic Metrics

```http
GET /v1/metrics/traffic
```

**Query Parameters**:

| Parameter | Type | Description |
|-----------|------|-------------|
| `backendId` | string | Filter by backend |
| `from` | timestamp | Start time |
| `to` | timestamp | End time |
| `granularity` | string | MINUTE, FIVE_MINUTES, HOUR, DAY |

**Response**:

```json
{
    "metrics": {
        "requestsTotal": 12345678,
        "requestsPerSecond": 1234,
        "bytesIn": 9876543210,
        "bytesOut": 8765432109,
        "bytesPerSecondIn": 1234567,
        "bytesPerSecondOut": 987654,
        "packetsIn": 87654321,
        "packetsOut": 76543210,
        "packetsPerSecond": 8765,
        "activeConnections": 12345,
        "newConnections": 234,
        "closedConnections": 232
    },
    "timestamp": "2024-01-15T14:30:00Z"
}
```

#### Get Attack Metrics

```http
GET /v1/metrics/attacks
```

**Response**:

```json
{
    "metrics": {
        "underAttack": true,
        "attackType": "SYN_FLOOD",
        "severity": "HIGH",
        "attackRequests": 1000000,
        "attackBytes": 50000000000,
        "attackPps": 50000,
        "attackBps": 500000000,
        "requestsDropped": 950000,
        "requestsChallenged": 25000,
        "requestsRateLimited": 25000,
        "uniqueAttackIps": 5432,
        "topSources": [
            {
                "ip": "185.220.101.42",
                "country": "DE",
                "asn": "AS12345",
                "requests": 50000,
                "bytes": 2500000000,
                "actionTaken": "DROP"
            }
        ]
    },
    "timestamp": "2024-01-15T14:30:00Z"
}
```

#### Get Time Series Metrics

```http
GET /v1/metrics/timeseries
```

**Query Parameters**:

| Parameter | Type | Description |
|-----------|------|-------------|
| `backendId` | string | Filter by backend |
| `metrics` | string[] | Metrics to fetch |
| `from` | timestamp | Start time |
| `to` | timestamp | End time |
| `granularity` | string | Data resolution |

**Response**:

```json
{
    "series": [
        {
            "metricName": "requests_per_second",
            "points": [
                {"timestamp": "2024-01-15T14:00:00Z", "value": 1234},
                {"timestamp": "2024-01-15T14:01:00Z", "value": 1256},
                {"timestamp": "2024-01-15T14:02:00Z", "value": 1198}
            ]
        }
    ]
}
```

#### Get Geographic Metrics

```http
GET /v1/metrics/geo
```

**Response**:

```json
{
    "countries": [
        {
            "countryCode": "US",
            "countryName": "United States",
            "requests": 5000000,
            "bytes": 250000000000,
            "uniqueIps": 50000,
            "blocked": false
        },
        {
            "countryCode": "CN",
            "countryName": "China",
            "requests": 1000000,
            "bytes": 50000000000,
            "uniqueIps": 100000,
            "blocked": true
        }
    ],
    "timestamp": "2024-01-15T14:30:00Z"
}
```

---

### IP Management

#### Block IP

```http
POST /v1/ips/block
Content-Type: application/json

{
    "ip": "1.2.3.4",
    "reason": "Manual block - suspicious activity",
    "durationSeconds": 3600,
    "backendId": "bk_xyz789"
}
```

**Response**:

```json
{
    "ip": "1.2.3.4",
    "reason": "Manual block - suspicious activity",
    "blockedAt": "2024-01-15T14:30:00Z",
    "expiresAt": "2024-01-15T15:30:00Z"
}
```

#### Unblock IP

```http
DELETE /v1/ips/block/{ip}
```

#### List Blocked IPs

```http
GET /v1/ips/blocked
```

**Query Parameters**:

| Parameter | Type | Description |
|-----------|------|-------------|
| `backendId` | string | Filter by backend |
| `page` | integer | Page number |
| `pageSize` | integer | Items per page |

#### Allow IP (Whitelist)

```http
POST /v1/ips/allow
Content-Type: application/json

{
    "ip": "10.0.0.0/8",
    "reason": "Internal network",
    "backendId": "bk_xyz789"
}
```

#### Remove from Whitelist

```http
DELETE /v1/ips/allow/{ip}
```

#### List Allowed IPs

```http
GET /v1/ips/allowed
```

---

### Organizations

#### Get Current Organization

```http
GET /v1/organizations/current
```

#### Update Organization

```http
PUT /v1/organizations/{orgId}
Content-Type: application/json

{
    "name": "My Company",
    "logoUrl": "https://example.com/logo.png"
}
```

#### Get Organization Usage

```http
GET /v1/organizations/{orgId}/usage
```

**Response**:

```json
{
    "backendsCount": 5,
    "domainsCount": 12,
    "filterRulesCount": 45,
    "bandwidthUsed": 5368709120000,
    "requestsUsed": 123456789,
    "limits": {
        "maxBackends": 10,
        "maxDomainsPerBackend": 5,
        "maxFilterRules": 100,
        "maxBandwidthBytes": 10995116277760,
        "maxRequests": 1000000000
    },
    "usageResetAt": "2024-02-01T00:00:00Z"
}
```

---

### Users

#### List Organization Members

```http
GET /v1/organizations/{orgId}/members
```

#### Invite Member

```http
POST /v1/organizations/{orgId}/invitations
Content-Type: application/json

{
    "email": "newuser@example.com",
    "role": "MEMBER"
}
```

**Roles**:

| Role | Description |
|------|-------------|
| `OWNER` | Full access, billing management |
| `ADMIN` | Full access except billing |
| `MEMBER` | Create/manage backends and rules |
| `VIEWER` | Read-only access |

#### Update Member Role

```http
PUT /v1/organizations/{orgId}/members/{userId}
Content-Type: application/json

{
    "role": "ADMIN"
}
```

#### Remove Member

```http
DELETE /v1/organizations/{orgId}/members/{userId}
```

---

### Alerts

#### List Alerts

```http
GET /v1/alerts
```

#### Create Alert

```http
POST /v1/alerts
Content-Type: application/json

{
    "backendId": "bk_xyz789",
    "name": "High Traffic Alert",
    "condition": {
        "metric": "requests_per_second",
        "operator": "GREATER_THAN",
        "threshold": 10000,
        "durationSeconds": 300
    },
    "notifications": [
        {
            "type": "EMAIL",
            "destination": "ops@example.com"
        },
        {
            "type": "SLACK",
            "destination": "https://hooks.slack.com/services/xxx"
        },
        {
            "type": "WEBHOOK",
            "destination": "https://your-service.com/alert"
        }
    ],
    "enabled": true
}
```

**Alert Operators**:

| Operator | Description |
|----------|-------------|
| `GREATER_THAN` | Value > threshold |
| `LESS_THAN` | Value < threshold |
| `EQUAL` | Value == threshold |
| `NOT_EQUAL` | Value != threshold |

**Notification Types**:

| Type | Description |
|------|-------------|
| `EMAIL` | Email notification |
| `WEBHOOK` | HTTP POST webhook |
| `SLACK` | Slack webhook |
| `DISCORD` | Discord webhook |
| `PAGERDUTY` | PagerDuty integration |

#### Update Alert

```http
PUT /v1/alerts/{alertId}
```

#### Delete Alert

```http
DELETE /v1/alerts/{alertId}
```

---

### Audit Logs

#### List Audit Logs

```http
GET /v1/audit-logs
```

**Query Parameters**:

| Parameter | Type | Description |
|-----------|------|-------------|
| `userId` | string | Filter by user |
| `resourceType` | string | Filter by resource type |
| `from` | timestamp | Start time |
| `to` | timestamp | End time |

**Response**:

```json
{
    "data": [
        {
            "id": "al_abc123",
            "organizationId": "org_xyz789",
            "userId": "usr_111222",
            "userEmail": "admin@example.com",
            "action": "backend.created",
            "resourceType": "backend",
            "resourceId": "bk_xyz789",
            "description": "Created backend 'mc.example.com'",
            "metadata": {
                "backendName": "mc.example.com",
                "backendType": "MINECRAFT_JAVA"
            },
            "ipAddress": "203.0.113.42",
            "userAgent": "Mozilla/5.0...",
            "timestamp": "2024-01-15T10:30:00Z"
        }
    ],
    "pagination": {
        "page": 1,
        "pageSize": 20,
        "totalCount": 156,
        "hasNext": true
    }
}
```

---

## gRPC API Reference

PistonProtection provides gRPC APIs for high-performance integrations.

### Connection

```
grpc://api.pistonprotection.io:9090
```

For TLS:
```
grpcs://api.pistonprotection.io:443
```

### Services

#### BackendService

```protobuf
service BackendService {
    rpc CreateBackend(CreateBackendRequest) returns (CreateBackendResponse);
    rpc GetBackend(GetBackendRequest) returns (GetBackendResponse);
    rpc UpdateBackend(UpdateBackendRequest) returns (UpdateBackendResponse);
    rpc DeleteBackend(DeleteBackendRequest) returns (DeleteBackendResponse);
    rpc ListBackends(ListBackendsRequest) returns (ListBackendsResponse);

    rpc AddOrigin(AddOriginRequest) returns (AddOriginResponse);
    rpc UpdateOrigin(UpdateOriginRequest) returns (UpdateOriginResponse);
    rpc RemoveOrigin(RemoveOriginRequest) returns (RemoveOriginResponse);

    rpc UpdateProtection(UpdateProtectionRequest) returns (UpdateProtectionResponse);
    rpc SetProtectionLevel(SetProtectionLevelRequest) returns (SetProtectionLevelResponse);

    rpc GetBackendStatus(GetBackendStatusRequest) returns (GetBackendStatusResponse);
    rpc WatchBackendStatus(WatchBackendStatusRequest) returns (stream BackendStatus);
}
```

#### FilterService

```protobuf
service FilterService {
    rpc CreateRule(CreateRuleRequest) returns (CreateRuleResponse);
    rpc GetRule(GetRuleRequest) returns (GetRuleResponse);
    rpc UpdateRule(UpdateRuleRequest) returns (UpdateRuleResponse);
    rpc DeleteRule(DeleteRuleRequest) returns (DeleteRuleResponse);
    rpc ListRules(ListRulesRequest) returns (ListRulesResponse);

    rpc BulkCreateRules(BulkCreateRulesRequest) returns (BulkCreateRulesResponse);
    rpc BulkDeleteRules(BulkDeleteRulesRequest) returns (BulkDeleteRulesResponse);
    rpc ReorderRules(ReorderRulesRequest) returns (ReorderRulesResponse);

    rpc GetRuleStats(GetRuleStatsRequest) returns (GetRuleStatsResponse);
    rpc WatchRules(WatchRulesRequest) returns (stream RuleUpdate);
}
```

#### MetricsService

```protobuf
service MetricsService {
    rpc GetTrafficMetrics(GetTrafficMetricsRequest) returns (GetTrafficMetricsResponse);
    rpc GetTrafficTimeSeries(TimeSeriesQuery) returns (GetTimeSeriesResponse);
    rpc StreamTrafficMetrics(StreamTrafficMetricsRequest) returns (stream TrafficMetrics);

    rpc GetAttackMetrics(GetAttackMetricsRequest) returns (GetAttackMetricsResponse);
    rpc StreamAttackMetrics(StreamAttackMetricsRequest) returns (stream AttackMetrics);

    rpc GetGeoMetrics(GetGeoMetricsRequest) returns (GetGeoMetricsResponse);

    rpc CreateAlert(CreateAlertRequest) returns (CreateAlertResponse);
    rpc ListAlerts(ListAlertsRequest) returns (ListAlertsResponse);
}
```

### Client Examples

#### Go Client

```go
package main

import (
    "context"
    "log"

    pb "github.com/pistonprotection/app/pkg/proto"
    "google.golang.org/grpc"
    "google.golang.org/grpc/credentials"
    "google.golang.org/grpc/metadata"
)

func main() {
    // Create TLS credentials
    creds, err := credentials.NewClientTLSFromFile("ca.crt", "")
    if err != nil {
        log.Fatal(err)
    }

    // Connect to gRPC server
    conn, err := grpc.Dial(
        "api.pistonprotection.io:443",
        grpc.WithTransportCredentials(creds),
    )
    if err != nil {
        log.Fatal(err)
    }
    defer conn.Close()

    // Create client
    client := pb.NewBackendServiceClient(conn)

    // Add authentication
    ctx := metadata.AppendToOutgoingContext(
        context.Background(),
        "authorization", "Bearer pp_live_xxxxxxxxxxxx",
    )

    // List backends
    resp, err := client.ListBackends(ctx, &pb.ListBackendsRequest{
        OrganizationId: "org_xyz789",
    })
    if err != nil {
        log.Fatal(err)
    }

    for _, backend := range resp.Backends {
        log.Printf("Backend: %s (%s)", backend.Name, backend.Status.Health)
    }

    // Stream metrics
    stream, err := client.WatchBackendStatus(ctx, &pb.WatchBackendStatusRequest{
        BackendId: "bk_abc123",
    })
    if err != nil {
        log.Fatal(err)
    }

    for {
        status, err := stream.Recv()
        if err != nil {
            break
        }
        log.Printf("Status: %v, RPS: %d", status.Health, status.RequestsPerSecond)
    }
}
```

#### Rust Client

```rust
use pistonprotection_proto::{
    backend_service_client::BackendServiceClient,
    ListBackendsRequest,
};
use tonic::{metadata::MetadataValue, transport::Channel, Request};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create channel with TLS
    let channel = Channel::from_static("https://api.pistonprotection.io:443")
        .tls_config(tonic::transport::ClientTlsConfig::new())?
        .connect()
        .await?;

    // Create client with auth interceptor
    let token: MetadataValue<_> = "Bearer pp_live_xxxxxxxxxxxx".parse()?;
    let mut client = BackendServiceClient::with_interceptor(
        channel,
        move |mut req: Request<()>| {
            req.metadata_mut().insert("authorization", token.clone());
            Ok(req)
        },
    );

    // List backends
    let response = client
        .list_backends(ListBackendsRequest {
            organization_id: "org_xyz789".into(),
            ..Default::default()
        })
        .await?;

    for backend in response.into_inner().backends {
        println!("Backend: {} ({:?})", backend.name, backend.status);
    }

    Ok(())
}
```

#### Python Client

```python
import grpc
from pistonprotection_proto import backend_pb2, backend_pb2_grpc

def main():
    # Create credentials
    credentials = grpc.ssl_channel_credentials()

    # Create channel
    channel = grpc.secure_channel(
        'api.pistonprotection.io:443',
        credentials
    )

    # Create stub
    stub = backend_pb2_grpc.BackendServiceStub(channel)

    # Create metadata with auth
    metadata = [('authorization', 'Bearer pp_live_xxxxxxxxxxxx')]

    # List backends
    request = backend_pb2.ListBackendsRequest(
        organization_id='org_xyz789'
    )
    response = stub.ListBackends(request, metadata=metadata)

    for backend in response.backends:
        print(f"Backend: {backend.name} ({backend.status.health})")

if __name__ == '__main__':
    main()
```

---

## Rate Limits

API rate limits vary by subscription tier:

| Tier | Requests/min | Burst | Concurrent Streams |
|------|--------------|-------|-------------------|
| Free | 60 | 10 | 1 |
| Pro | 600 | 100 | 10 |
| Business | 3000 | 500 | 50 |
| Enterprise | Unlimited | - | Unlimited |

### Rate Limit Headers

```http
X-RateLimit-Limit: 600
X-RateLimit-Remaining: 542
X-RateLimit-Reset: 1705330800
X-RateLimit-RetryAfter: 45
```

### Handling Rate Limits

When rate limited, you'll receive a `429 Too Many Requests` response:

```json
{
    "type": "https://api.pistonprotection.io/errors/rate-limited",
    "title": "Rate Limited",
    "status": 429,
    "detail": "You have exceeded the rate limit. Please retry after 45 seconds.",
    "retryAfter": 45
}
```

Implement exponential backoff:

```python
import time
import requests

def api_request_with_retry(url, max_retries=5):
    for attempt in range(max_retries):
        response = requests.get(url, headers={'Authorization': 'Bearer ...'})

        if response.status_code == 429:
            retry_after = int(response.headers.get('X-RateLimit-RetryAfter', 60))
            time.sleep(retry_after)
            continue

        return response

    raise Exception("Max retries exceeded")
```

---

## Error Handling

### Error Response Format

All errors follow RFC 7807 (Problem Details for HTTP APIs):

```json
{
    "type": "https://api.pistonprotection.io/errors/validation",
    "title": "Validation Error",
    "status": 400,
    "detail": "The IP address '1.2.3.4.5' is not valid",
    "instance": "/v1/filters",
    "errors": [
        {
            "field": "match.sourceIps[0]",
            "code": "invalid_ip",
            "message": "Invalid IP address format"
        }
    ]
}
```

### Error Types

| Type | Status | Description |
|------|--------|-------------|
| `validation` | 400 | Invalid request parameters |
| `authentication` | 401 | Missing or invalid credentials |
| `authorization` | 403 | Insufficient permissions |
| `not-found` | 404 | Resource not found |
| `conflict` | 409 | Resource conflict |
| `rate-limited` | 429 | Rate limit exceeded |
| `internal` | 500 | Internal server error |
| `unavailable` | 503 | Service temporarily unavailable |

### Common Error Codes

| Code | Description |
|------|-------------|
| `invalid_ip` | Invalid IP address format |
| `invalid_cidr` | Invalid CIDR notation |
| `invalid_port` | Port number out of range |
| `duplicate_name` | Resource name already exists |
| `limit_exceeded` | Subscription limit exceeded |
| `backend_not_found` | Backend does not exist |
| `rule_not_found` | Filter rule does not exist |

---

## Webhooks

Configure webhooks to receive real-time event notifications.

### Creating a Webhook

```http
POST /v1/webhooks
Content-Type: application/json

{
    "url": "https://your-service.com/webhook",
    "events": [
        "attack.detected",
        "attack.mitigated",
        "backend.health_changed",
        "backend.under_attack",
        "rule.triggered",
        "alert.fired"
    ],
    "secret": "whsec_xxxxxxxxxxxxxxxxxxxx",
    "enabled": true
}
```

### Event Types

| Event | Description |
|-------|-------------|
| `attack.detected` | New DDoS attack detected |
| `attack.mitigated` | Attack successfully mitigated |
| `attack.escalated` | Attack severity increased |
| `backend.created` | New backend created |
| `backend.updated` | Backend configuration changed |
| `backend.deleted` | Backend deleted |
| `backend.health_changed` | Backend health status changed |
| `backend.under_attack` | Backend is under attack |
| `rule.created` | Filter rule created |
| `rule.triggered` | Filter rule matched traffic |
| `alert.fired` | Alert condition triggered |
| `alert.resolved` | Alert condition resolved |

### Webhook Payload

```json
{
    "id": "evt_abc123def456",
    "type": "attack.detected",
    "timestamp": "2024-01-15T14:30:00Z",
    "organizationId": "org_xyz789",
    "data": {
        "backendId": "bk_abc123",
        "backendName": "mc.example.com",
        "attackType": "SYN_FLOOD",
        "severity": "HIGH",
        "attackPps": 500000,
        "attackBps": 5000000000,
        "uniqueSources": 5432,
        "topCountries": ["CN", "RU", "US"],
        "mitigationStatus": "ACTIVE"
    }
}
```

### Signature Verification

Webhooks include a signature header for verification:

```
X-PistonProtection-Signature: sha256=xxxxxxxxxxxxxxxx
X-PistonProtection-Timestamp: 1705330800
```

**Verification example (Python)**:

```python
import hmac
import hashlib
import time

def verify_webhook(payload: bytes, signature: str, timestamp: str, secret: str) -> bool:
    # Check timestamp is recent (within 5 minutes)
    current_time = int(time.time())
    webhook_time = int(timestamp)
    if abs(current_time - webhook_time) > 300:
        return False

    # Compute expected signature
    signed_payload = f"{timestamp}.{payload.decode()}"
    expected_sig = hmac.new(
        secret.encode(),
        signed_payload.encode(),
        hashlib.sha256
    ).hexdigest()

    # Compare signatures
    return hmac.compare_digest(f"sha256={expected_sig}", signature)
```

### Webhook Retry Policy

Failed webhooks are retried with exponential backoff:

| Attempt | Delay |
|---------|-------|
| 1 | Immediate |
| 2 | 1 minute |
| 3 | 5 minutes |
| 4 | 30 minutes |
| 5 | 2 hours |
| 6 | 8 hours |
| 7 | 24 hours |

After 7 failed attempts, the webhook is disabled.

---

## SDKs and Examples

### Official SDKs

- **Go**: [github.com/pistonprotection/go-sdk](https://github.com/pistonprotection/go-sdk)
- **Rust**: [crates.io/pistonprotection](https://crates.io/pistonprotection)
- **Python**: [pypi.org/project/pistonprotection](https://pypi.org/project/pistonprotection/)
- **Node.js**: [npmjs.com/package/@pistonprotection/sdk](https://npmjs.com/package/@pistonprotection/sdk)

### Quick Start Examples

**Python SDK**:

```python
from pistonprotection import Client

client = Client(api_key="pp_live_xxxxxxxxxxxx")

# List backends
backends = client.backends.list()

# Create filter rule
rule = client.filters.create(
    backend_id="bk_xyz789",
    name="Block bad IPs",
    match={"source_ips": ["1.2.3.4"]},
    action="DROP"
)

# Get metrics
metrics = client.metrics.get_traffic(backend_id="bk_xyz789")
```

**Node.js SDK**:

```javascript
const PistonProtection = require('@pistonprotection/sdk');

const client = new PistonProtection({ apiKey: 'pp_live_xxxxxxxxxxxx' });

// List backends
const backends = await client.backends.list();

// Stream attack metrics
const stream = client.metrics.streamAttacks({ backendId: 'bk_xyz789' });
stream.on('data', (metrics) => {
    console.log('Attack status:', metrics.underAttack);
});
```

---

## Related Documentation

- [Configuration Reference](configuration.md) - Helm and service configuration
- [Protocol Filters](filters.md) - Protocol-specific filtering
- [Operations Guide](operations.md) - Monitoring and maintenance
