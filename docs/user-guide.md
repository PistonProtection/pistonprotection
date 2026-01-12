# User Guide

This guide walks you through using PistonProtection to protect your applications and services from DDoS attacks.

## Table of Contents

- [Getting Started](#getting-started)
- [Dashboard Overview](#dashboard-overview)
- [Managing Backends](#managing-backends)
- [Configuring Protection](#configuring-protection)
- [Understanding Metrics](#understanding-metrics)
- [Filter Rules](#filter-rules)
- [Alerts and Notifications](#alerts-and-notifications)
- [Account Management](#account-management)
- [Billing and Subscriptions](#billing-and-subscriptions)
- [FAQ](#faq)

---

## Getting Started

### First Login

1. Navigate to your PistonProtection dashboard URL (e.g., `https://dashboard.pistonprotection.example.com`)
2. Enter your credentials provided by your administrator
3. On first login, you'll be prompted to:
   - Change your password
   - Set up two-factor authentication (recommended)
   - Review and accept the terms of service

### Initial Setup Wizard

The setup wizard guides you through:

1. **Organization Setup** - Configure your organization name and settings
2. **First Backend** - Add your first protected service
3. **DNS Configuration** - Point your DNS to PistonProtection
4. **Verify Protection** - Confirm traffic is flowing through the system

### Quick Start

**Step 1: Add a Backend**

```
Name: my-web-app
Domain: app.example.com
Origin: 203.0.113.10:443
Protocol: HTTPS
```

**Step 2: Update DNS**

Change your DNS A record for `app.example.com` to point to:
```
pistonprotection-ingress.example.com
```

Or use the provided IP addresses for your region.

**Step 3: Verify**

Visit your domain and check the dashboard to see traffic flowing.

---

## Dashboard Overview

### Main Dashboard

The main dashboard provides an at-a-glance view of your protection status.

#### Traffic Overview Panel

| Metric | Description |
|--------|-------------|
| Requests/sec | Current request rate |
| Bandwidth | Current throughput in Gbps |
| Blocked | Requests blocked in last hour |
| Attack Status | Current attack detection status |

#### Status Indicators

| Indicator | Meaning |
|-----------|---------|
| Green | All systems healthy, normal traffic |
| Yellow | Elevated traffic or minor issues |
| Orange | Attack detected, mitigation active |
| Red | Under heavy attack or system issue |

### Navigation

| Section | Description |
|---------|-------------|
| **Dashboard** | Overview and quick stats |
| **Backends** | Manage protected services |
| **Filter Rules** | Configure filtering rules |
| **Analytics** | Detailed traffic analysis |
| **Alerts** | View and manage alerts |
| **Settings** | Account and organization settings |

---

## Managing Backends

### What is a Backend?

A backend represents a service you want to protect. It consists of:
- **Domain** - The public domain users access
- **Origins** - Your actual servers that handle requests
- **Protection Settings** - How aggressively to filter traffic

### Creating a Backend

#### Basic Configuration

1. Navigate to **Backends** > **Add Backend**
2. Enter basic information:

| Field | Description | Example |
|-------|-------------|---------|
| Name | Friendly name | My Web App |
| Domain | Public domain | app.example.com |
| Protocol | HTTP, HTTPS, TCP, UDP | HTTPS |

#### Adding Origins

Origins are your actual servers:

1. Click **Add Origin**
2. Configure:

| Field | Description | Example |
|-------|-------------|---------|
| Address | Server IP or hostname | 203.0.113.10 |
| Port | Service port | 443 |
| Weight | Load balancing weight | 100 |
| Priority | Failover priority | 1 |

#### Load Balancing

Choose a load balancing method:

| Method | Description | Best For |
|--------|-------------|----------|
| Round Robin | Distribute equally | Uniform servers |
| Weighted | Distribute by weight | Mixed capacity servers |
| Least Connections | Send to least busy | Long-lived connections |
| IP Hash | Same client to same server | Session affinity |
| Random | Random selection | Simple distribution |

### Health Checks

Configure how PistonProtection monitors your origins:

```yaml
Health Check Settings:
  Protocol: HTTP
  Path: /health
  Interval: 30 seconds
  Timeout: 5 seconds
  Healthy Threshold: 2 consecutive successes
  Unhealthy Threshold: 3 consecutive failures
```

#### Health Check Types

| Type | Use Case | Configuration |
|------|----------|---------------|
| HTTP | Web services | Path, expected status code |
| HTTPS | Secure web services | Path, certificate validation |
| TCP | TCP services | Port connectivity |
| gRPC | gRPC services | Service name, method |

### Backend Status

| Status | Meaning |
|--------|---------|
| Healthy | All origins responding |
| Degraded | Some origins unhealthy |
| Unhealthy | All origins failing |
| Maintenance | Manually disabled |

---

## Configuring Protection

### Protection Levels

Choose the appropriate protection level for your needs:

| Level | Description | Use Case |
|-------|-------------|----------|
| **Off** | No filtering | Testing only |
| **Low** | Basic filtering | Low-risk services |
| **Medium** | Standard filtering | Most services |
| **High** | Aggressive filtering | High-value targets |
| **Under Attack** | Maximum filtering | Active attack |

### Protection Settings

#### Rate Limiting

Control the rate of requests from individual IPs:

```yaml
Rate Limiting:
  Requests per Second: 100
  Burst Size: 200
  Window: 60 seconds
  Action: Challenge  # or Block
```

**Configuration Options:**

| Setting | Description | Recommended |
|---------|-------------|-------------|
| Requests/sec | Maximum sustained rate | 50-200 |
| Burst | Maximum burst size | 2x requests/sec |
| Window | Time window for counting | 60 seconds |
| Action | What happens when exceeded | Challenge |

#### Challenge Settings

When suspicious traffic is detected:

| Challenge Type | Description | Difficulty |
|----------------|-------------|------------|
| JavaScript | Simple JS execution | Low |
| CAPTCHA | Human verification | Medium |
| Proof of Work | Computational puzzle | High |

#### Bot Protection

Protect against automated threats:

```yaml
Bot Protection:
  Enabled: true
  Good Bots: Allow
  Bad Bots: Block
  Unknown Bots: Challenge

  Known Good Bots:
    - Googlebot
    - Bingbot
    - Slackbot
```

### Protocol-Specific Settings

#### HTTP/HTTPS Protection

| Setting | Description |
|---------|-------------|
| WAF Rules | Web Application Firewall |
| Header Validation | Check HTTP headers |
| Body Inspection | Inspect request bodies |
| Path Filtering | Block malicious paths |

#### TCP/UDP Protection

| Setting | Description |
|---------|-------------|
| SYN Flood Protection | Detect SYN floods |
| Amplification Blocking | Block amplification attacks |
| Port Scanning Detection | Detect port scans |

#### Game Server Protection (Minecraft)

| Setting | Description |
|---------|-------------|
| Protocol Validation | Verify Minecraft protocol |
| Player Verification | Verify real players |
| Query Protection | Protect query port |
| Anti-Bot | Block bot join attempts |

### GeoIP Blocking

Block or allow traffic by country:

```yaml
GeoIP Rules:
  Mode: Allow List  # or Block List
  Countries:
    - US
    - CA
    - GB
  Action: Block
  Exceptions:
    - IP: 203.0.113.0/24
      Reason: Office network
```

---

## Understanding Metrics

### Traffic Metrics

#### Requests

| Metric | Description |
|--------|-------------|
| Total Requests | All requests received |
| Allowed Requests | Requests passed to origin |
| Blocked Requests | Requests blocked by filters |
| Challenged Requests | Requests that received challenges |

#### Bandwidth

| Metric | Description |
|--------|-------------|
| Inbound | Traffic received |
| Outbound | Traffic sent to clients |
| Origin | Traffic sent to origins |

### Attack Metrics

#### Attack Types

| Type | Description | Indicators |
|------|-------------|------------|
| Volumetric | High volume traffic | Gbps spike |
| Protocol | Protocol exploitation | Invalid packets |
| Application | L7 attacks | Slow requests |

#### Attack Dashboard

The attack dashboard shows:

1. **Active Attacks** - Currently detected attacks
2. **Attack Timeline** - Historical attack patterns
3. **Top Attack Sources** - Countries and IPs
4. **Mitigation Stats** - Blocked traffic volume

### Reading Charts

#### Traffic Timeline

```
Traffic (requests/sec)
|
|    ####
|   ######  Attack mitigated
|  ########
| ##########
|############  Normal traffic
+------------->  Time
```

#### Geographic Distribution

The map view shows:
- Green: Normal traffic regions
- Yellow: Elevated traffic
- Red: Attack sources

### Custom Analytics

Create custom dashboards:

1. Go to **Analytics** > **Custom Dashboards**
2. Click **New Dashboard**
3. Add widgets:
   - Time series charts
   - Pie charts
   - Tables
   - Counters

#### Example Custom Query

```sql
SELECT
  time_bucket('5 minutes', timestamp) AS time,
  COUNT(*) AS requests,
  COUNT(*) FILTER (WHERE blocked = true) AS blocked
FROM traffic_logs
WHERE backend_id = 'backend-123'
  AND timestamp > NOW() - INTERVAL '24 hours'
GROUP BY time
ORDER BY time
```

---

## Filter Rules

### Rule Types

| Type | Description | Layer |
|------|-------------|-------|
| IP Rules | Block/allow specific IPs | L3 |
| Rate Rules | Limit request rates | L4 |
| Header Rules | Filter by HTTP headers | L7 |
| Path Rules | Filter by URL paths | L7 |
| Custom Rules | Complex matching logic | L3-L7 |

### Creating Rules

#### Simple IP Block

1. Go to **Filter Rules** > **Add Rule**
2. Select **IP Block**
3. Configure:

```yaml
Name: Block known attacker
Type: IP Block
Source: 198.51.100.0/24
Action: Block
Duration: Permanent
```

#### Rate Limit Rule

```yaml
Name: API rate limit
Type: Rate Limit
Match:
  Path: /api/*
Limit:
  Requests: 100
  Window: 60 seconds
Action: Block
```

#### HTTP Header Rule

```yaml
Name: Block bad user agents
Type: Header Match
Match:
  Header: User-Agent
  Pattern: ".*curl.*|.*wget.*"
  Regex: true
Action: Challenge
```

### Rule Priority

Rules are evaluated in priority order (lower number = higher priority):

| Priority | Rule Type | Example |
|----------|-----------|---------|
| 1-100 | Allowlist | Known good IPs |
| 101-500 | Critical blocks | Known attackers |
| 501-1000 | Rate limits | API protection |
| 1001-5000 | General rules | Bot protection |
| 5001+ | Default rules | Catch-all |

### Rule Templates

Use templates for common scenarios:

| Template | Description |
|----------|-------------|
| API Protection | Rate limits for API endpoints |
| Login Protection | Protect authentication endpoints |
| WordPress Security | Block common WP attacks |
| Game Server | Minecraft/game specific rules |

### Testing Rules

Before enabling rules:

1. Set rule to **Log Only** mode
2. Review matched traffic in logs
3. Adjust rule if needed
4. Switch to **Active** mode

---

## Alerts and Notifications

### Alert Types

| Alert | Trigger | Severity |
|-------|---------|----------|
| Attack Started | Attack detected | Critical |
| Attack Ended | Attack stopped | Info |
| Backend Unhealthy | All origins down | Critical |
| Backend Degraded | Some origins down | Warning |
| High Traffic | Traffic spike | Warning |
| Certificate Expiring | Cert expires soon | Warning |

### Notification Channels

#### Email

```yaml
Email Notification:
  Recipients:
    - ops@example.com
    - security@example.com
  Severity: Warning and above
  Digest: Hourly summary
```

#### Slack

```yaml
Slack Notification:
  Webhook: https://hooks.slack.com/...
  Channel: #security-alerts
  Severity: Critical only
  Include:
    - Attack details
    - Affected backends
    - Recommended actions
```

#### PagerDuty

```yaml
PagerDuty:
  Service Key: xxx-xxx-xxx
  Severity Mapping:
    Critical: P1
    Warning: P3
    Info: Suppress
```

#### Webhook

```yaml
Custom Webhook:
  URL: https://api.example.com/alerts
  Method: POST
  Headers:
    Authorization: Bearer xxx
  Payload Template: |
    {
      "alert": "{{ .AlertName }}",
      "severity": "{{ .Severity }}",
      "backend": "{{ .Backend }}",
      "details": "{{ .Details }}"
    }
```

### Managing Alerts

#### Acknowledge Alerts

Click **Acknowledge** to:
- Silence repeated notifications
- Indicate someone is investigating
- Keep alert visible in dashboard

#### Resolve Alerts

Alerts auto-resolve when:
- Attack ends
- Backend recovers
- Issue is fixed

Or manually resolve with a note.

#### Alert History

View past alerts in **Alerts** > **History**:
- Filter by date range
- Filter by severity
- Export to CSV

---

## Account Management

### User Roles

| Role | Permissions |
|------|-------------|
| **Owner** | Full access, billing, delete org |
| **Admin** | Full access except billing |
| **Editor** | Manage backends and rules |
| **Viewer** | Read-only access |
| **API Only** | API access only |

### Inviting Users

1. Go to **Settings** > **Team**
2. Click **Invite User**
3. Enter email and select role
4. User receives invitation email

### Two-Factor Authentication

Enable 2FA for enhanced security:

1. Go to **Settings** > **Security**
2. Click **Enable 2FA**
3. Scan QR code with authenticator app
4. Enter verification code
5. Save backup codes securely

### API Keys

Manage API keys for programmatic access:

| Key Type | Use Case | Permissions |
|----------|----------|-------------|
| Read Only | Monitoring, dashboards | GET requests |
| Read/Write | Automation, CI/CD | All requests |
| Admin | Full management | All + user management |

#### Creating an API Key

1. Go to **Settings** > **API Keys**
2. Click **Generate New Key**
3. Select permissions
4. Set expiration (optional)
5. Copy and store the key securely

**Important:** API keys are shown only once. Store them securely.

### Audit Log

View all account activity:

| Event | Details Logged |
|-------|----------------|
| Login | User, IP, device, location |
| Config Change | What changed, who changed it |
| Rule Update | Rule details, before/after |
| Alert Action | Acknowledge, resolve actions |

---

## Billing and Subscriptions

### Plans

| Plan | Traffic | Backends | Features |
|------|---------|----------|----------|
| **Starter** | 10 Gbps | 5 | Basic protection |
| **Professional** | 50 Gbps | 25 | Advanced features |
| **Enterprise** | Unlimited | Unlimited | Premium support |

### Usage Metrics

Track your usage:

| Metric | Description | Billing Impact |
|--------|-------------|----------------|
| Peak Traffic | Highest Gbps | Plan limit |
| Total Transfer | Monthly GB | Overage charges |
| Backends | Active backends | Plan limit |
| Requests | Monthly requests | Some plans |

### Viewing Invoices

1. Go to **Settings** > **Billing**
2. View current usage
3. Download past invoices
4. Update payment method

### Upgrading Plans

1. Go to **Settings** > **Billing** > **Change Plan**
2. Select new plan
3. Review prorated charges
4. Confirm upgrade

Upgrades take effect immediately.

### Overage Charges

If you exceed your plan limits:

| Resource | Overage Rate |
|----------|--------------|
| Traffic | $0.05/GB over limit |
| Backends | $20/backend over limit |

Configure overage alerts:
1. Go to **Settings** > **Billing** > **Alerts**
2. Set threshold (e.g., 80% of limit)
3. Choose notification method

---

## FAQ

### General Questions

**Q: How quickly does protection take effect?**

A: Protection is active within seconds of DNS propagation. During an attack, mitigation begins immediately upon detection (typically <1 second).

**Q: Does PistonProtection inspect my traffic?**

A: We inspect packet headers and metadata for filtering. For HTTPS, we perform TLS termination only if you enable it and upload your certificates. Otherwise, we proxy encrypted traffic without inspection.

**Q: What happens if PistonProtection goes down?**

A: We have multiple points of presence and automatic failover. If a PoP becomes unavailable, traffic automatically routes to the next nearest healthy PoP. For self-hosted deployments, configure high availability as described in the Operations Guide.

### Configuration Questions

**Q: How do I protect multiple domains?**

A: Create a separate backend for each domain. Each backend can have its own protection settings and origin servers.

**Q: Can I use PistonProtection with a CDN?**

A: Yes. Configure your CDN as the origin in PistonProtection, or place PistonProtection in front of your CDN depending on your needs.

**Q: How do I whitelist my own services?**

A: Create an IP allowlist rule with high priority:
```yaml
Name: Internal Services
Type: IP Allow
Source: 10.0.0.0/8
Priority: 1
```

### Protection Questions

**Q: Why is legitimate traffic being blocked?**

A: This usually happens when:
1. Protection level is too high - try lowering it
2. Rate limits are too strict - increase thresholds
3. Bot detection false positive - add to allowlist

Check the analytics to see what rules are triggering.

**Q: What types of attacks can PistonProtection stop?**

A: We protect against:
- Volumetric attacks (UDP floods, ICMP floods)
- Protocol attacks (SYN floods, fragmentation)
- Application attacks (HTTP floods, Slowloris)
- Game-specific attacks (Minecraft crashes, join floods)
- Amplification attacks (DNS, NTP, memcached)

**Q: How do I know if I'm being attacked?**

A: Signs of an attack:
- Alert notifications
- Orange/Red status in dashboard
- Spike in blocked traffic
- Increased latency to origin

### Troubleshooting

**Q: My site is slow after enabling PistonProtection**

A: Check:
1. Origin server health - are health checks passing?
2. Geographic distance - is your origin close to users?
3. Challenge rate - are too many users being challenged?
4. Protocol - is HTTP/2 or HTTP/3 enabled?

**Q: I can't access my site through PistonProtection**

A: Verify:
1. DNS is pointing to PistonProtection
2. Backend status is healthy
3. Origin server is accessible
4. SSL certificates are valid (for HTTPS)

**Q: API requests are being blocked**

A: For API traffic:
1. Identify your API by path or header
2. Create a rule to allow or increase limits
3. Use an API key in requests for identification

### Contact Support

If you need additional help:

- **Documentation**: https://docs.pistonprotection.io
- **Community Forum**: https://community.pistonprotection.io
- **Email Support**: support@pistonprotection.io
- **Enterprise Support**: Available 24/7 for Enterprise plans

When contacting support, include:
- Organization ID
- Backend ID
- Time of issue
- Error messages or screenshots
- Steps to reproduce

---

## Related Documentation

- [Installation Guide](installation.md) - Deployment instructions
- [Configuration Reference](configuration.md) - All configuration options
- [API Documentation](api.md) - API reference
- [Filter Documentation](filters.md) - Protocol filter details
- [Operations Guide](operations.md) - Operational procedures
