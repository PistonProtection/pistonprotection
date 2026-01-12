/**
 * Test utilities and helpers for React component testing
 */

import React, { ReactElement, ReactNode } from 'react';
import { render, RenderOptions, RenderResult } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';

// ============================================================================
// Test Query Client
// ============================================================================

/**
 * Creates a query client configured for testing
 */
export function createTestQueryClient(): QueryClient {
  return new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        gcTime: Infinity,
        staleTime: Infinity,
      },
      mutations: {
        retry: false,
      },
    },
  });
}

// ============================================================================
// Test Providers
// ============================================================================

interface TestProvidersProps {
  children: ReactNode;
  queryClient?: QueryClient;
}

/**
 * Wrapper component that provides all necessary context providers for testing
 */
export function TestProviders({
  children,
  queryClient = createTestQueryClient(),
}: TestProvidersProps): ReactElement {
  return (
    <QueryClientProvider client={queryClient}>
      {children}
    </QueryClientProvider>
  );
}

// ============================================================================
// Custom Render Functions
// ============================================================================

interface CustomRenderOptions extends Omit<RenderOptions, 'wrapper'> {
  queryClient?: QueryClient;
}

/**
 * Custom render function that wraps components with necessary providers
 */
export function renderWithProviders(
  ui: ReactElement,
  options: CustomRenderOptions = {}
): RenderResult & { queryClient: QueryClient } {
  const { queryClient = createTestQueryClient(), ...renderOptions } = options;

  const Wrapper = ({ children }: { children: ReactNode }) => (
    <TestProviders queryClient={queryClient}>{children}</TestProviders>
  );

  return {
    ...render(ui, { wrapper: Wrapper, ...renderOptions }),
    queryClient,
  };
}

// ============================================================================
// Mock Data Factories
// ============================================================================

export interface MockBackend {
  id: string;
  name: string;
  address: string;
  port: number;
  protocol: 'tcp' | 'udp' | 'http' | 'https';
  status: 'healthy' | 'degraded' | 'offline';
  enabled: boolean;
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

export function createMockBackend(overrides: Partial<MockBackend> = {}): MockBackend {
  const id = overrides.id || `backend-${Math.random().toString(36).substr(2, 9)}`;
  return {
    id,
    name: 'Test Backend',
    address: '192.168.1.100',
    port: 25565,
    protocol: 'tcp',
    status: 'healthy',
    enabled: true,
    stats: {
      requests: 10000,
      blocked: 500,
      passed: 9500,
      latency: 15,
      bytesIn: 1000000,
      bytesOut: 2000000,
    },
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
    ...overrides,
  };
}

export interface MockFilterRule {
  id: string;
  name: string;
  description?: string;
  type: 'ip' | 'geo' | 'rate' | 'pattern' | 'protocol' | 'custom';
  action: 'drop' | 'ratelimit' | 'allow' | 'log' | 'challenge';
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

export function createMockFilterRule(overrides: Partial<MockFilterRule> = {}): MockFilterRule {
  const id = overrides.id || `rule-${Math.random().toString(36).substr(2, 9)}`;
  return {
    id,
    name: 'Test Rule',
    description: 'A test filter rule',
    type: 'ip',
    action: 'drop',
    priority: 50,
    enabled: true,
    config: {
      ips: ['192.168.1.0/24'],
    },
    matches: 100,
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
    ...overrides,
  };
}

export interface MockMetrics {
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

export function createMockMetrics(overrides: Partial<MockMetrics> = {}): MockMetrics {
  return {
    totalRequests: 100000,
    blockedRequests: 5000,
    passedRequests: 95000,
    challengedRequests: 1000,
    bytesIn: 10000000,
    bytesOut: 20000000,
    avgLatency: 15,
    p50Latency: 10,
    p95Latency: 50,
    p99Latency: 100,
    activeConnections: 500,
    peakConnections: 1000,
    timestamp: new Date().toISOString(),
    ...overrides,
  };
}

export interface MockUser {
  id: string;
  email: string;
  name?: string;
  image?: string;
  emailVerified: boolean;
  createdAt: string;
}

export function createMockUser(overrides: Partial<MockUser> = {}): MockUser {
  const id = overrides.id || `user-${Math.random().toString(36).substr(2, 9)}`;
  return {
    id,
    email: 'test@example.com',
    name: 'Test User',
    emailVerified: true,
    createdAt: new Date().toISOString(),
    ...overrides,
  };
}

export interface MockOrganization {
  id: string;
  name: string;
  slug: string;
  logo?: string;
  createdAt: string;
}

export function createMockOrganization(overrides: Partial<MockOrganization> = {}): MockOrganization {
  const id = overrides.id || `org-${Math.random().toString(36).substr(2, 9)}`;
  return {
    id,
    name: 'Test Organization',
    slug: 'test-org',
    createdAt: new Date().toISOString(),
    ...overrides,
  };
}

export interface MockSubscription {
  id: string;
  plan: 'free' | 'starter' | 'pro' | 'enterprise';
  status: 'active' | 'canceled' | 'past_due' | 'trialing';
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

export function createMockSubscription(overrides: Partial<MockSubscription> = {}): MockSubscription {
  const id = overrides.id || `sub-${Math.random().toString(36).substr(2, 9)}`;
  return {
    id,
    plan: 'pro',
    status: 'active',
    currentPeriodStart: new Date().toISOString(),
    currentPeriodEnd: new Date(Date.now() + 30 * 24 * 60 * 60 * 1000).toISOString(),
    cancelAtPeriodEnd: false,
    limits: {
      backends: 10,
      requestsPerMonth: 10000000,
      filterRules: 100,
      retentionDays: 90,
    },
    usage: {
      backends: 3,
      requestsThisMonth: 2500000,
      filterRules: 15,
    },
    ...overrides,
  };
}

export interface MockApiKey {
  id: string;
  name: string;
  key: string;
  prefix: string;
  createdAt: string;
  lastUsed?: string;
}

export function createMockApiKey(overrides: Partial<MockApiKey> = {}): MockApiKey {
  const id = overrides.id || `key-${Math.random().toString(36).substr(2, 9)}`;
  return {
    id,
    name: 'Test API Key',
    key: `pp_live_${Math.random().toString(36).substr(2, 24)}`,
    prefix: 'pp_live_xxxx',
    createdAt: new Date().toISOString(),
    ...overrides,
  };
}

// ============================================================================
// Mock API Response Helpers
// ============================================================================

export function mockFetch(data: unknown, options: { status?: number; ok?: boolean } = {}) {
  const { status = 200, ok = true } = options;
  return vi.fn().mockResolvedValue({
    ok,
    status,
    json: () => Promise.resolve(data),
    text: () => Promise.resolve(JSON.stringify(data)),
  });
}

export function mockFetchError(message: string, status = 500) {
  return vi.fn().mockResolvedValue({
    ok: false,
    status,
    json: () => Promise.resolve({ message }),
    text: () => Promise.resolve(JSON.stringify({ message })),
  });
}

export function mockFetchNetworkError() {
  return vi.fn().mockRejectedValue(new Error('Network error'));
}

// ============================================================================
// Wait Helpers
// ============================================================================

/**
 * Wait for a condition to be true
 */
export async function waitFor(
  condition: () => boolean | Promise<boolean>,
  options: { timeout?: number; interval?: number } = {}
): Promise<void> {
  const { timeout = 5000, interval = 50 } = options;
  const start = Date.now();

  while (Date.now() - start < timeout) {
    if (await condition()) {
      return;
    }
    await new Promise(resolve => setTimeout(resolve, interval));
  }

  throw new Error('Condition not met within timeout');
}

// ============================================================================
// Re-export testing library utilities
// ============================================================================

export * from '@testing-library/react';
export { default as userEvent } from '@testing-library/user-event';
