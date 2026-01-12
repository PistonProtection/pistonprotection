import { queryOptions } from "@tanstack/react-query"

// API Base URL - configure this based on environment
const API_BASE_URL = import.meta.env.VITE_API_URL || "http://localhost:8080/api"

// Types
export interface User {
  id: string
  email: string
  name: string
  avatar?: string
  plan: "free" | "pro" | "enterprise"
  createdAt: string
}

export interface Backend {
  id: string
  name: string
  host: string
  port: number
  protocol: "http" | "https" | "tcp" | "udp"
  healthCheck: {
    enabled: boolean
    path?: string
    interval: number
  }
  status: "healthy" | "unhealthy" | "unknown"
  requestsPerSecond: number
  latency: number
  createdAt: string
  updatedAt: string
}

export interface FilterRule {
  id: string
  name: string
  description: string
  type: "rate_limit" | "ip_block" | "geo_block" | "header_filter" | "custom"
  enabled: boolean
  priority: number
  conditions: FilterCondition[]
  action: "allow" | "block" | "challenge" | "log"
  createdAt: string
  updatedAt: string
}

export interface FilterCondition {
  field: string
  operator: "eq" | "neq" | "contains" | "regex" | "gt" | "lt" | "in"
  value: string | string[] | number
}

export interface AnalyticsData {
  timestamp: string
  requests: number
  blocked: number
  allowed: number
  challenged: number
  bandwidth: number
  latencyP50: number
  latencyP95: number
  latencyP99: number
}

export interface DashboardMetrics {
  totalRequests: number
  blockedRequests: number
  allowedRequests: number
  challengedRequests: number
  activeBackends: number
  totalBackends: number
  activeFilters: number
  avgLatency: number
  bandwidth: number
  attacksBlocked: number
  requestsChange: number
  blockedChange: number
  bandwidthChange: number
  latencyChange: number
}

export interface BillingInfo {
  plan: "free" | "pro" | "enterprise"
  status: "active" | "past_due" | "canceled"
  currentPeriodStart: string
  currentPeriodEnd: string
  usage: {
    requests: number
    requestsLimit: number
    bandwidth: number
    bandwidthLimit: number
    backends: number
    backendsLimit: number
    filters: number
    filtersLimit: number
  }
  invoices: Invoice[]
}

export interface Invoice {
  id: string
  date: string
  amount: number
  status: "paid" | "pending" | "failed"
  downloadUrl: string
}

export interface Settings {
  notifications: {
    email: boolean
    slack: boolean
    webhook: boolean
    attackAlerts: boolean
    weeklyReports: boolean
  }
  security: {
    twoFactorEnabled: boolean
    apiKeyRotation: number
    ipWhitelist: string[]
  }
  general: {
    timezone: string
    dateFormat: string
    theme: "light" | "dark" | "system"
  }
}

// API Client
class ApiClient {
  private baseUrl: string
  private token: string | null = null

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl
  }

  setToken(token: string | null) {
    this.token = token
  }

  private async fetch<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const headers: HeadersInit = {
      "Content-Type": "application/json",
      ...options.headers,
    }

    if (this.token) {
      ;(headers as Record<string, string>)["Authorization"] =
        `Bearer ${this.token}`
    }

    const response = await fetch(`${this.baseUrl}${endpoint}`, {
      ...options,
      headers,
    })

    if (!response.ok) {
      const error = await response.json().catch(() => ({}))
      throw new Error(error.message || `API Error: ${response.status}`)
    }

    return response.json()
  }

  // Auth
  async login(email: string, password: string) {
    return this.fetch<{ token: string; user: User }>("/auth/login", {
      method: "POST",
      body: JSON.stringify({ email, password }),
    })
  }

  async register(email: string, password: string, name: string) {
    return this.fetch<{ token: string; user: User }>("/auth/register", {
      method: "POST",
      body: JSON.stringify({ email, password, name }),
    })
  }

  async logout() {
    return this.fetch<void>("/auth/logout", { method: "POST" })
  }

  async getUser() {
    return this.fetch<User>("/auth/me")
  }

  // Dashboard
  async getDashboardMetrics() {
    return this.fetch<DashboardMetrics>("/dashboard/metrics")
  }

  // Backends
  async getBackends() {
    return this.fetch<Backend[]>("/backends")
  }

  async getBackend(id: string) {
    return this.fetch<Backend>(`/backends/${id}`)
  }

  async createBackend(data: Omit<Backend, "id" | "createdAt" | "updatedAt" | "status" | "requestsPerSecond" | "latency">) {
    return this.fetch<Backend>("/backends", {
      method: "POST",
      body: JSON.stringify(data),
    })
  }

  async updateBackend(id: string, data: Partial<Backend>) {
    return this.fetch<Backend>(`/backends/${id}`, {
      method: "PATCH",
      body: JSON.stringify(data),
    })
  }

  async deleteBackend(id: string) {
    return this.fetch<void>(`/backends/${id}`, { method: "DELETE" })
  }

  // Filters
  async getFilters() {
    return this.fetch<FilterRule[]>("/filters")
  }

  async getFilter(id: string) {
    return this.fetch<FilterRule>(`/filters/${id}`)
  }

  async createFilter(data: Omit<FilterRule, "id" | "createdAt" | "updatedAt">) {
    return this.fetch<FilterRule>("/filters", {
      method: "POST",
      body: JSON.stringify(data),
    })
  }

  async updateFilter(id: string, data: Partial<FilterRule>) {
    return this.fetch<FilterRule>(`/filters/${id}`, {
      method: "PATCH",
      body: JSON.stringify(data),
    })
  }

  async deleteFilter(id: string) {
    return this.fetch<void>(`/filters/${id}`, { method: "DELETE" })
  }

  async toggleFilter(id: string, enabled: boolean) {
    return this.fetch<FilterRule>(`/filters/${id}/toggle`, {
      method: "POST",
      body: JSON.stringify({ enabled }),
    })
  }

  // Analytics
  async getAnalytics(params: {
    startDate: string
    endDate: string
    granularity: "minute" | "hour" | "day"
  }) {
    const searchParams = new URLSearchParams(params as Record<string, string>)
    return this.fetch<AnalyticsData[]>(`/analytics?${searchParams}`)
  }

  // Billing
  async getBilling() {
    return this.fetch<BillingInfo>("/billing")
  }

  async updatePlan(plan: "free" | "pro" | "enterprise") {
    return this.fetch<BillingInfo>("/billing/plan", {
      method: "POST",
      body: JSON.stringify({ plan }),
    })
  }

  // Settings
  async getSettings() {
    return this.fetch<Settings>("/settings")
  }

  async updateSettings(data: Partial<Settings>) {
    return this.fetch<Settings>("/settings", {
      method: "PATCH",
      body: JSON.stringify(data),
    })
  }
}

export const apiClient = new ApiClient(API_BASE_URL)

// Mock data generators for development
function generateMockMetrics(): DashboardMetrics {
  return {
    totalRequests: 1_234_567,
    blockedRequests: 45_678,
    allowedRequests: 1_188_889,
    challengedRequests: 12_345,
    activeBackends: 4,
    totalBackends: 5,
    activeFilters: 12,
    avgLatency: 23,
    bandwidth: 1.5 * 1024 * 1024 * 1024, // 1.5 GB
    attacksBlocked: 892,
    requestsChange: 12.5,
    blockedChange: -8.3,
    bandwidthChange: 5.2,
    latencyChange: -15.0,
  }
}

function generateMockBackends(): Backend[] {
  return [
    {
      id: "1",
      name: "Primary API Server",
      host: "api.example.com",
      port: 443,
      protocol: "https",
      healthCheck: { enabled: true, path: "/health", interval: 30 },
      status: "healthy",
      requestsPerSecond: 1250,
      latency: 12,
      createdAt: "2024-01-15T10:00:00Z",
      updatedAt: "2024-01-20T15:30:00Z",
    },
    {
      id: "2",
      name: "Secondary API Server",
      host: "api2.example.com",
      port: 443,
      protocol: "https",
      healthCheck: { enabled: true, path: "/health", interval: 30 },
      status: "healthy",
      requestsPerSecond: 890,
      latency: 15,
      createdAt: "2024-01-16T10:00:00Z",
      updatedAt: "2024-01-20T15:30:00Z",
    },
    {
      id: "3",
      name: "WebSocket Server",
      host: "ws.example.com",
      port: 8080,
      protocol: "tcp",
      healthCheck: { enabled: true, path: "/ws/health", interval: 15 },
      status: "healthy",
      requestsPerSecond: 450,
      latency: 8,
      createdAt: "2024-01-17T10:00:00Z",
      updatedAt: "2024-01-20T15:30:00Z",
    },
    {
      id: "4",
      name: "Static Assets CDN",
      host: "cdn.example.com",
      port: 443,
      protocol: "https",
      healthCheck: { enabled: true, path: "/", interval: 60 },
      status: "healthy",
      requestsPerSecond: 3200,
      latency: 5,
      createdAt: "2024-01-18T10:00:00Z",
      updatedAt: "2024-01-20T15:30:00Z",
    },
    {
      id: "5",
      name: "Legacy Backend",
      host: "legacy.example.com",
      port: 80,
      protocol: "http",
      healthCheck: { enabled: false, interval: 60 },
      status: "unhealthy",
      requestsPerSecond: 0,
      latency: 0,
      createdAt: "2024-01-10T10:00:00Z",
      updatedAt: "2024-01-20T15:30:00Z",
    },
  ]
}

function generateMockFilters(): FilterRule[] {
  return [
    {
      id: "1",
      name: "Rate Limit - API Endpoints",
      description: "Limit requests to 100/min per IP for API endpoints",
      type: "rate_limit",
      enabled: true,
      priority: 1,
      conditions: [{ field: "path", operator: "contains", value: "/api/" }],
      action: "block",
      createdAt: "2024-01-15T10:00:00Z",
      updatedAt: "2024-01-20T15:30:00Z",
    },
    {
      id: "2",
      name: "Block Known Bad IPs",
      description: "Block traffic from known malicious IP ranges",
      type: "ip_block",
      enabled: true,
      priority: 0,
      conditions: [
        { field: "ip", operator: "in", value: ["192.168.1.0/24", "10.0.0.0/8"] },
      ],
      action: "block",
      createdAt: "2024-01-14T10:00:00Z",
      updatedAt: "2024-01-20T15:30:00Z",
    },
    {
      id: "3",
      name: "Geo Block - High Risk Countries",
      description: "Challenge traffic from high-risk geographical regions",
      type: "geo_block",
      enabled: true,
      priority: 2,
      conditions: [{ field: "country", operator: "in", value: ["XX", "YY"] }],
      action: "challenge",
      createdAt: "2024-01-13T10:00:00Z",
      updatedAt: "2024-01-20T15:30:00Z",
    },
    {
      id: "4",
      name: "Block SQL Injection Attempts",
      description: "Block requests containing SQL injection patterns",
      type: "custom",
      enabled: true,
      priority: 0,
      conditions: [
        {
          field: "query",
          operator: "regex",
          value: "(?i)(union|select|insert|drop|delete|update)",
        },
      ],
      action: "block",
      createdAt: "2024-01-12T10:00:00Z",
      updatedAt: "2024-01-20T15:30:00Z",
    },
    {
      id: "5",
      name: "Bot Detection",
      description: "Challenge requests with suspicious user agents",
      type: "header_filter",
      enabled: false,
      priority: 3,
      conditions: [
        { field: "user-agent", operator: "regex", value: "(bot|crawler|spider)" },
      ],
      action: "challenge",
      createdAt: "2024-01-11T10:00:00Z",
      updatedAt: "2024-01-20T15:30:00Z",
    },
  ]
}

function generateMockAnalytics(
  startDate: string,
  endDate: string,
  granularity: "minute" | "hour" | "day"
): AnalyticsData[] {
  const start = new Date(startDate)
  const end = new Date(endDate)
  const data: AnalyticsData[] = []

  const increment =
    granularity === "minute" ? 60000 : granularity === "hour" ? 3600000 : 86400000

  for (let time = start.getTime(); time <= end.getTime(); time += increment) {
    const baseRequests = 10000 + Math.random() * 5000
    const blocked = Math.floor(baseRequests * (0.03 + Math.random() * 0.02))
    const challenged = Math.floor(baseRequests * (0.01 + Math.random() * 0.01))
    const allowed = Math.floor(baseRequests - blocked - challenged)

    data.push({
      timestamp: new Date(time).toISOString(),
      requests: Math.floor(baseRequests),
      blocked,
      allowed,
      challenged,
      bandwidth: Math.floor(baseRequests * 1024 * (0.5 + Math.random() * 0.5)),
      latencyP50: 10 + Math.random() * 10,
      latencyP95: 30 + Math.random() * 20,
      latencyP99: 80 + Math.random() * 40,
    })
  }

  return data
}

function generateMockBilling(): BillingInfo {
  return {
    plan: "pro",
    status: "active",
    currentPeriodStart: "2024-01-01T00:00:00Z",
    currentPeriodEnd: "2024-02-01T00:00:00Z",
    usage: {
      requests: 8_500_000,
      requestsLimit: 10_000_000,
      bandwidth: 450 * 1024 * 1024 * 1024,
      bandwidthLimit: 500 * 1024 * 1024 * 1024,
      backends: 5,
      backendsLimit: 10,
      filters: 12,
      filtersLimit: 50,
    },
    invoices: [
      {
        id: "inv_001",
        date: "2024-01-01T00:00:00Z",
        amount: 99,
        status: "paid",
        downloadUrl: "/invoices/inv_001.pdf",
      },
      {
        id: "inv_002",
        date: "2023-12-01T00:00:00Z",
        amount: 99,
        status: "paid",
        downloadUrl: "/invoices/inv_002.pdf",
      },
      {
        id: "inv_003",
        date: "2023-11-01T00:00:00Z",
        amount: 99,
        status: "paid",
        downloadUrl: "/invoices/inv_003.pdf",
      },
    ],
  }
}

function generateMockSettings(): Settings {
  return {
    notifications: {
      email: true,
      slack: true,
      webhook: false,
      attackAlerts: true,
      weeklyReports: true,
    },
    security: {
      twoFactorEnabled: true,
      apiKeyRotation: 90,
      ipWhitelist: ["192.168.1.0/24"],
    },
    general: {
      timezone: "America/New_York",
      dateFormat: "MM/DD/YYYY",
      theme: "system",
    },
  }
}

// Query Options with mock data
export const dashboardMetricsOptions = queryOptions({
  queryKey: ["dashboard", "metrics"],
  queryFn: async () => {
    // In development, return mock data
    if (import.meta.env.DEV) {
      await new Promise((r) => setTimeout(r, 500))
      return generateMockMetrics()
    }
    return apiClient.getDashboardMetrics()
  },
  staleTime: 30000,
  refetchInterval: 60000,
})

export const backendsOptions = queryOptions({
  queryKey: ["backends"],
  queryFn: async () => {
    if (import.meta.env.DEV) {
      await new Promise((r) => setTimeout(r, 500))
      return generateMockBackends()
    }
    return apiClient.getBackends()
  },
  staleTime: 30000,
})

export const filtersOptions = queryOptions({
  queryKey: ["filters"],
  queryFn: async () => {
    if (import.meta.env.DEV) {
      await new Promise((r) => setTimeout(r, 500))
      return generateMockFilters()
    }
    return apiClient.getFilters()
  },
  staleTime: 30000,
})

export const analyticsOptions = (params: {
  startDate: string
  endDate: string
  granularity: "minute" | "hour" | "day"
}) =>
  queryOptions({
    queryKey: ["analytics", params],
    queryFn: async () => {
      if (import.meta.env.DEV) {
        await new Promise((r) => setTimeout(r, 800))
        return generateMockAnalytics(
          params.startDate,
          params.endDate,
          params.granularity
        )
      }
      return apiClient.getAnalytics(params)
    },
    staleTime: 60000,
  })

export const billingOptions = queryOptions({
  queryKey: ["billing"],
  queryFn: async () => {
    if (import.meta.env.DEV) {
      await new Promise((r) => setTimeout(r, 500))
      return generateMockBilling()
    }
    return apiClient.getBilling()
  },
  staleTime: 300000,
})

export const settingsOptions = queryOptions({
  queryKey: ["settings"],
  queryFn: async () => {
    if (import.meta.env.DEV) {
      await new Promise((r) => setTimeout(r, 500))
      return generateMockSettings()
    }
    return apiClient.getSettings()
  },
  staleTime: 300000,
})

export const userOptions = queryOptions({
  queryKey: ["user"],
  queryFn: async () => {
    if (import.meta.env.DEV) {
      await new Promise((r) => setTimeout(r, 300))
      return {
        id: "1",
        email: "user@example.com",
        name: "John Doe",
        avatar: undefined,
        plan: "pro" as const,
        createdAt: "2024-01-01T00:00:00Z",
      }
    }
    return apiClient.getUser()
  },
  staleTime: 300000,
})
