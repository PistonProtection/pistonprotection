import { queryOptions } from "@tanstack/react-query";

const API_BASE_URL = import.meta.env.VITE_API_URL || "http://localhost:3001";

// Types
export interface Backend {
  id: string;
  name: string;
  address: string;
  port: number;
  protocol: "tcp" | "udp" | "http" | "https";
  status: "healthy" | "degraded" | "offline";
  enabled: boolean;
  healthCheck?: {
    interval: number;
    timeout: number;
    path?: string;
  };
  stats: {
    requests: number;
    blocked: number;
    passed: number;
    latency: number;
    bytesIn: number;
    bytesOut: number;
  };
  createdAt: string;
  updatedAt: string;
}

export interface FilterRule {
  id: string;
  name: string;
  description?: string;
  type: "ip" | "geo" | "rate" | "pattern" | "protocol" | "custom";
  action: "drop" | "ratelimit" | "allow" | "log" | "challenge";
  priority: number;
  enabled: boolean;
  backendId?: string;
  config: {
    ips?: string[];
    countries?: string[];
    rateLimit?: {
      requests: number;
      window: number;
    };
    pattern?: string;
    protocol?: string;
    expression?: string;
  };
  matches: number;
  createdAt: string;
  updatedAt: string;
}

export interface Metrics {
  totalRequests: number;
  blockedRequests: number;
  passedRequests: number;
  challengedRequests: number;
  bytesIn: number;
  bytesOut: number;
  avgLatency: number;
  p50Latency: number;
  p95Latency: number;
  p99Latency: number;
  activeConnections: number;
  peakConnections: number;
  timestamp: string;
}

export interface MetricsHistory {
  data: Metrics[];
  interval: string;
  from: string;
  to: string;
}

export interface TrafficEvent {
  id: string;
  timestamp: string;
  sourceIp: string;
  sourcePort: number;
  destIp: string;
  destPort: number;
  protocol: string;
  action: string;
  ruleId?: string;
  ruleName?: string;
  backendId?: string;
  backendName?: string;
  bytes: number;
  country?: string;
  asn?: string;
}

export interface Subscription {
  id: string;
  plan: "free" | "starter" | "pro" | "enterprise";
  status: "active" | "canceled" | "past_due" | "trialing";
  currentPeriodStart: string;
  currentPeriodEnd: string;
  cancelAtPeriodEnd: boolean;
  limits: {
    backends: number;
    requestsPerMonth: number;
    filterRules: number;
    retentionDays: number;
  };
  usage: {
    backends: number;
    requestsThisMonth: number;
    filterRules: number;
  };
}

export interface Invoice {
  id: string;
  amount: number;
  currency: string;
  status: "draft" | "open" | "paid" | "void" | "uncollectible" | "pending";
  date: string;
  description: string;
  createdAt: string;
  paidAt?: string;
  invoicePdf?: string;
}

export interface ApiKey {
  id: string;
  name: string;
  key: string;
  prefix: string;
  createdAt: string;
  lastUsed?: string;
}

export interface SubscriptionDetail {
  id: string;
  status: "active" | "canceling" | "canceled" | "past_due" | "trialing";
  plan: {
    id: string;
    name: string;
    price: number;
    backends: string;
    protection: string;
  };
  usage: {
    backends: number;
    requests: string;
    dataTransferred: string;
  };
  nextBillingDate: string;
  paymentMethod?: {
    brand: string;
    last4: string;
    expiry: string;
  };
  invoices?: Invoice[];
}

export interface OrganizationDetail {
  id: string;
  name: string;
  slug: string;
  logo?: string;
  createdAt: string;
}

export interface OrganizationMemberDetail {
  id: string;
  role: "owner" | "admin" | "member";
  user: {
    id: string;
    name?: string;
    email: string;
    image?: string;
  };
  joinedAt: string;
}

export interface AnalyticsData {
  timeRange: string;
  data: Array<{
    timestamp: string;
    requests: number;
    blocked: number;
    passed: number;
    bandwidth: number;
    latency: number;
  }>;
  topCountries: Array<{
    code: string;
    name: string;
    requests: number;
    percentage: number;
  }>;
  attackTypes: Array<{
    type: string;
    count: number;
    percentage: number;
  }>;
}

export interface Organization {
  id: string;
  name: string;
  slug: string;
  logo?: string;
  createdAt: string;
  members: OrganizationMember[];
}

export interface OrganizationMember {
  id: string;
  userId: string;
  email: string;
  name?: string;
  role: "owner" | "admin" | "member" | "viewer";
  joinedAt: string;
}

// API Client
class ApiClient {
  private baseUrl: string;

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl;
  }

  private async fetch<T>(
    path: string,
    options: RequestInit = {}
  ): Promise<T> {
    const headers: HeadersInit = {
      "Content-Type": "application/json",
      ...options.headers,
    };

    const response = await fetch(`${this.baseUrl}${path}`, {
      ...options,
      headers,
      credentials: "include",
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ message: "Unknown error" }));
      throw new Error(error.message || `HTTP ${response.status}`);
    }

    if (response.status === 204) {
      return undefined as T;
    }

    return response.json();
  }

  // Backends
  async listBackends(): Promise<Backend[]> {
    return this.fetch<Backend[]>("/api/v1/backends");
  }

  async getBackend(id: string): Promise<Backend> {
    return this.fetch<Backend>(`/api/v1/backends/${id}`);
  }

  async createBackend(data: Omit<Backend, "id" | "status" | "stats" | "createdAt" | "updatedAt">): Promise<Backend> {
    return this.fetch<Backend>("/api/v1/backends", {
      method: "POST",
      body: JSON.stringify(data),
    });
  }

  async updateBackend(id: string, data: Partial<Backend>): Promise<Backend> {
    return this.fetch<Backend>(`/api/v1/backends/${id}`, {
      method: "PATCH",
      body: JSON.stringify(data),
    });
  }

  async deleteBackend(id: string): Promise<void> {
    await this.fetch<void>(`/api/v1/backends/${id}`, {
      method: "DELETE",
    });
  }

  async toggleBackend(id: string, enabled: boolean): Promise<Backend> {
    return this.fetch<Backend>(`/api/v1/backends/${id}/toggle`, {
      method: "POST",
      body: JSON.stringify({ enabled }),
    });
  }

  // Filter Rules
  async listFilterRules(backendId?: string): Promise<FilterRule[]> {
    const path = backendId
      ? `/api/v1/filters?backendId=${backendId}`
      : "/api/v1/filters";
    return this.fetch<FilterRule[]>(path);
  }

  async getFilterRule(id: string): Promise<FilterRule> {
    return this.fetch<FilterRule>(`/api/v1/filters/${id}`);
  }

  async createFilterRule(data: Omit<FilterRule, "id" | "matches" | "createdAt" | "updatedAt">): Promise<FilterRule> {
    return this.fetch<FilterRule>("/api/v1/filters", {
      method: "POST",
      body: JSON.stringify(data),
    });
  }

  async updateFilterRule(id: string, data: Partial<FilterRule>): Promise<FilterRule> {
    return this.fetch<FilterRule>(`/api/v1/filters/${id}`, {
      method: "PATCH",
      body: JSON.stringify(data),
    });
  }

  async deleteFilterRule(id: string): Promise<void> {
    await this.fetch<void>(`/api/v1/filters/${id}`, {
      method: "DELETE",
    });
  }

  async toggleFilterRule(id: string, enabled: boolean): Promise<FilterRule> {
    return this.fetch<FilterRule>(`/api/v1/filters/${id}/toggle`, {
      method: "POST",
      body: JSON.stringify({ enabled }),
    });
  }

  async reorderFilterRules(ruleIds: string[]): Promise<FilterRule[]> {
    return this.fetch<FilterRule[]>("/api/v1/filters/reorder", {
      method: "POST",
      body: JSON.stringify({ ruleIds }),
    });
  }

  // Metrics
  async getMetrics(backendId?: string): Promise<Metrics> {
    const path = backendId
      ? `/api/v1/metrics?backendId=${backendId}`
      : "/api/v1/metrics";
    return this.fetch<Metrics>(path);
  }

  async getMetricsHistory(
    backendId?: string,
    from?: string,
    to?: string,
    interval?: string
  ): Promise<MetricsHistory> {
    const params = new URLSearchParams();
    if (backendId) params.append("backendId", backendId);
    if (from) params.append("from", from);
    if (to) params.append("to", to);
    if (interval) params.append("interval", interval);
    return this.fetch<MetricsHistory>(`/api/v1/metrics/history?${params}`);
  }

  // Traffic Events (real-time)
  async getRecentEvents(limit = 100): Promise<TrafficEvent[]> {
    return this.fetch<TrafficEvent[]>(`/api/v1/events?limit=${limit}`);
  }

  subscribeToEvents(
    onEvent: (event: TrafficEvent) => void,
    onError?: (error: Event) => void
  ): EventSource {
    const eventSource = new EventSource(`${this.baseUrl}/api/v1/events/stream`, {
      withCredentials: true,
    });

    eventSource.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        onEvent(data);
      } catch (e) {
        console.error("Failed to parse event:", e);
      }
    };

    if (onError) {
      eventSource.onerror = onError;
    }

    return eventSource;
  }

  // Subscription & Billing
  async getSubscription(): Promise<Subscription> {
    return this.fetch<Subscription>("/api/v1/billing/subscription");
  }

  async createCheckoutSession(plan: string): Promise<{ url: string }> {
    return this.fetch<{ url: string }>("/api/v1/billing/checkout", {
      method: "POST",
      body: JSON.stringify({ plan }),
    });
  }

  async createPortalSession(): Promise<{ url: string }> {
    return this.fetch<{ url: string }>("/api/v1/billing/portal", {
      method: "POST",
    });
  }

  async getInvoices(): Promise<Invoice[]> {
    return this.fetch<Invoice[]>("/api/v1/billing/invoices");
  }

  async cancelSubscription(): Promise<Subscription> {
    return this.fetch<Subscription>("/api/v1/billing/cancel", {
      method: "POST",
    });
  }

  // Settings
  async getSettings(): Promise<Record<string, unknown>> {
    return this.fetch<Record<string, unknown>>("/api/v1/settings");
  }

  async updateSettings(settings: Record<string, unknown>): Promise<Record<string, unknown>> {
    return this.fetch<Record<string, unknown>>("/api/v1/settings", {
      method: "PATCH",
      body: JSON.stringify(settings),
    });
  }

  // API Keys
  async listApiKeys(): Promise<Array<{ id: string; name: string; prefix: string; createdAt: string; lastUsedAt?: string }>> {
    return this.fetch("/api/v1/api-keys");
  }

  async createApiKey(name: string): Promise<{ id: string; key: string; name: string }> {
    return this.fetch("/api/v1/api-keys", {
      method: "POST",
      body: JSON.stringify({ name }),
    });
  }

  async deleteApiKey(id: string): Promise<void> {
    await this.fetch(`/api/v1/api-keys/${id}`, {
      method: "DELETE",
    });
  }
}

export const apiClient = new ApiClient(API_BASE_URL);

// React Query Options
export const backendsQueryOptions = () =>
  queryOptions({
    queryKey: ["backends"],
    queryFn: () => apiClient.listBackends(),
    staleTime: 30000,
  });

export const backendQueryOptions = (id: string) =>
  queryOptions({
    queryKey: ["backends", id],
    queryFn: () => apiClient.getBackend(id),
    enabled: !!id,
  });

export const filterRulesQueryOptions = (backendId?: string) =>
  queryOptions({
    queryKey: ["filterRules", backendId],
    queryFn: () => apiClient.listFilterRules(backendId),
    staleTime: 30000,
  });

export const filterRuleQueryOptions = (id: string) =>
  queryOptions({
    queryKey: ["filterRules", "detail", id],
    queryFn: () => apiClient.getFilterRule(id),
    enabled: !!id,
  });

export const metricsQueryOptions = (backendId?: string) =>
  queryOptions({
    queryKey: ["metrics", backendId],
    queryFn: () => apiClient.getMetrics(backendId),
    refetchInterval: 5000,
  });

export const metricsHistoryQueryOptions = (
  backendId?: string,
  from?: string,
  to?: string,
  interval?: string
) =>
  queryOptions({
    queryKey: ["metricsHistory", backendId, from, to, interval],
    queryFn: () => apiClient.getMetricsHistory(backendId, from, to, interval),
    staleTime: 60000,
  });

export const recentEventsQueryOptions = (limit = 100) =>
  queryOptions({
    queryKey: ["events", "recent", limit],
    queryFn: () => apiClient.getRecentEvents(limit),
    refetchInterval: 10000,
  });

export const subscriptionQueryOptions = () =>
  queryOptions({
    queryKey: ["subscription"],
    queryFn: async () => {
      // Mock data for development - in production, call apiClient.getSubscription()
      return {
        id: "sub_1",
        status: "active",
        plan: {
          id: "pro",
          name: "Pro",
          price: 99,
          backends: "5",
          protection: "100 Gbps",
        },
        usage: {
          backends: 3,
          requests: "2.1M",
          dataTransferred: "42 GB",
        },
        nextBillingDate: new Date(Date.now() + 15 * 24 * 60 * 60 * 1000).toISOString(),
        paymentMethod: {
          brand: "Visa",
          last4: "4242",
          expiry: "12/25",
        },
        invoices: [
          {
            id: "inv_1",
            amount: 99,
            currency: "usd",
            status: "paid",
            date: new Date(Date.now() - 15 * 24 * 60 * 60 * 1000).toISOString(),
            description: "Pro Plan - Monthly",
            createdAt: new Date(Date.now() - 15 * 24 * 60 * 60 * 1000).toISOString(),
          },
          {
            id: "inv_2",
            amount: 99,
            currency: "usd",
            status: "paid",
            date: new Date(Date.now() - 45 * 24 * 60 * 60 * 1000).toISOString(),
            description: "Pro Plan - Monthly",
            createdAt: new Date(Date.now() - 45 * 24 * 60 * 60 * 1000).toISOString(),
          },
        ],
      } as SubscriptionDetail;
    },
    staleTime: 60000,
  });

export const invoicesQueryOptions = () =>
  queryOptions({
    queryKey: ["invoices"],
    queryFn: () => apiClient.getInvoices(),
    staleTime: 300000,
  });

export const settingsQueryOptions = () =>
  queryOptions({
    queryKey: ["settings"],
    queryFn: () => apiClient.getSettings(),
    staleTime: 60000,
  });

export const apiKeysQueryOptions = () =>
  queryOptions({
    queryKey: ["api-keys"],
    queryFn: async () => {
      // Mock data for development
      return [
        {
          id: "1",
          name: "Production Key",
          key: "pp_live_xxxx1234567890abcdef",
          prefix: "pp_live_xxxx",
          createdAt: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000).toISOString(),
          lastUsed: new Date(Date.now() - 2 * 60 * 60 * 1000).toISOString(),
        },
        {
          id: "2",
          name: "Development Key",
          key: "pp_test_yyyy0987654321fedcba",
          prefix: "pp_test_yyyy",
          createdAt: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000).toISOString(),
          lastUsed: undefined,
        },
      ] as ApiKey[];
    },
    staleTime: 60000,
  });

export const organizationQueryOptions = () =>
  queryOptions({
    queryKey: ["organization"],
    queryFn: async () => {
      // Mock data for development
      return {
        id: "org_1",
        name: "Acme Inc",
        slug: "acme",
        createdAt: new Date(Date.now() - 90 * 24 * 60 * 60 * 1000).toISOString(),
      } as OrganizationDetail;
    },
    staleTime: 60000,
  });

export const organizationMembersQueryOptions = () =>
  queryOptions({
    queryKey: ["organization-members"],
    queryFn: async () => {
      // Mock data for development
      return [
        {
          id: "member_1",
          role: "owner",
          user: {
            id: "user_1",
            name: "John Doe",
            email: "john@example.com",
          },
          joinedAt: new Date(Date.now() - 90 * 24 * 60 * 60 * 1000).toISOString(),
        },
        {
          id: "member_2",
          role: "admin",
          user: {
            id: "user_2",
            name: "Jane Smith",
            email: "jane@example.com",
          },
          joinedAt: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000).toISOString(),
        },
        {
          id: "member_3",
          role: "member",
          user: {
            id: "user_3",
            name: "Bob Wilson",
            email: "bob@example.com",
          },
          joinedAt: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000).toISOString(),
        },
      ] as OrganizationMemberDetail[];
    },
    staleTime: 60000,
  });

export const analyticsQueryOptions = (timeRange: string = "24h") =>
  queryOptions({
    queryKey: ["analytics", timeRange],
    queryFn: async () => {
      // In production, this would fetch from the API
      return {
        timeRange,
        data: [],
        topCountries: [],
        attackTypes: [],
      } as AnalyticsData;
    },
    staleTime: 30000,
  });
