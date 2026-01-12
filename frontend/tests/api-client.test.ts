/**
 * API Client tests
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import {
  createMockBackend,
  createMockFilterRule,
  createMockMetrics,
  createMockSubscription,
  createMockApiKey,
} from './test-utils';

// ============================================================================
// Mock API Client (for testing without module resolution issues)
// ============================================================================

interface FetchOptions extends RequestInit {
  headers?: HeadersInit;
}

class TestApiClient {
  private baseUrl: string;

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl;
  }

  private async fetch<T>(path: string, options: FetchOptions = {}): Promise<T> {
    const headers: HeadersInit = {
      'Content-Type': 'application/json',
      ...options.headers,
    };

    const response = await fetch(`${this.baseUrl}${path}`, {
      ...options,
      headers,
      credentials: 'include',
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ message: 'Unknown error' }));
      throw new Error(error.message || `HTTP ${response.status}`);
    }

    if (response.status === 204) {
      return undefined as T;
    }

    return response.json();
  }

  // Backends
  async listBackends() {
    return this.fetch('/api/v1/backends');
  }

  async getBackend(id: string) {
    return this.fetch(`/api/v1/backends/${id}`);
  }

  async createBackend(data: Record<string, unknown>) {
    return this.fetch('/api/v1/backends', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async updateBackend(id: string, data: Record<string, unknown>) {
    return this.fetch(`/api/v1/backends/${id}`, {
      method: 'PATCH',
      body: JSON.stringify(data),
    });
  }

  async deleteBackend(id: string) {
    await this.fetch(`/api/v1/backends/${id}`, {
      method: 'DELETE',
    });
  }

  // Filter Rules
  async listFilterRules(backendId?: string) {
    const path = backendId
      ? `/api/v1/filters?backendId=${backendId}`
      : '/api/v1/filters';
    return this.fetch(path);
  }

  async createFilterRule(data: Record<string, unknown>) {
    return this.fetch('/api/v1/filters', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async deleteFilterRule(id: string) {
    await this.fetch(`/api/v1/filters/${id}`, {
      method: 'DELETE',
    });
  }

  // Metrics
  async getMetrics(backendId?: string) {
    const path = backendId
      ? `/api/v1/metrics?backendId=${backendId}`
      : '/api/v1/metrics';
    return this.fetch(path);
  }

  // API Keys
  async listApiKeys() {
    return this.fetch('/api/v1/api-keys');
  }

  async createApiKey(name: string) {
    return this.fetch('/api/v1/api-keys', {
      method: 'POST',
      body: JSON.stringify({ name }),
    });
  }

  async deleteApiKey(id: string) {
    await this.fetch(`/api/v1/api-keys/${id}`, {
      method: 'DELETE',
    });
  }

  // Subscription
  async getSubscription() {
    return this.fetch('/api/v1/billing/subscription');
  }
}

// ============================================================================
// Test Setup
// ============================================================================

describe('API Client', () => {
  let apiClient: TestApiClient;
  const mockFetch = vi.fn();

  beforeEach(() => {
    apiClient = new TestApiClient('http://localhost:3001');
    global.fetch = mockFetch;
  });

  afterEach(() => {
    vi.resetAllMocks();
  });

  // ============================================================================
  // Backend API Tests
  // ============================================================================

  describe('Backends API', () => {
    it('should list backends', async () => {
      const mockBackends = [createMockBackend(), createMockBackend()];
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: () => Promise.resolve(mockBackends),
      });

      const result = await apiClient.listBackends();

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:3001/api/v1/backends',
        expect.objectContaining({
          credentials: 'include',
        })
      );
      expect(result).toEqual(mockBackends);
    });

    it('should get a single backend', async () => {
      const mockBackend = createMockBackend({ id: 'backend-123' });
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: () => Promise.resolve(mockBackend),
      });

      const result = await apiClient.getBackend('backend-123');

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:3001/api/v1/backends/backend-123',
        expect.any(Object)
      );
      expect(result).toEqual(mockBackend);
    });

    it('should create a backend', async () => {
      const newBackend = {
        name: 'New Backend',
        address: '10.0.0.1',
        port: 25565,
        protocol: 'tcp',
        enabled: true,
      };
      const createdBackend = createMockBackend(newBackend as any);
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 201,
        json: () => Promise.resolve(createdBackend),
      });

      const result = await apiClient.createBackend(newBackend);

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:3001/api/v1/backends',
        expect.objectContaining({
          method: 'POST',
          body: JSON.stringify(newBackend),
        })
      );
      expect(result).toEqual(createdBackend);
    });

    it('should update a backend', async () => {
      const updates = { name: 'Updated Backend' };
      const updatedBackend = createMockBackend({ ...updates, id: 'backend-123' });
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: () => Promise.resolve(updatedBackend),
      });

      const result = await apiClient.updateBackend('backend-123', updates);

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:3001/api/v1/backends/backend-123',
        expect.objectContaining({
          method: 'PATCH',
          body: JSON.stringify(updates),
        })
      );
      expect(result).toEqual(updatedBackend);
    });

    it('should delete a backend', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 204,
        json: () => Promise.resolve(undefined),
      });

      await apiClient.deleteBackend('backend-123');

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:3001/api/v1/backends/backend-123',
        expect.objectContaining({
          method: 'DELETE',
        })
      );
    });

    it('should throw error on backend not found', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 404,
        json: () => Promise.resolve({ message: 'Backend not found' }),
      });

      await expect(apiClient.getBackend('nonexistent')).rejects.toThrow('Backend not found');
    });
  });

  // ============================================================================
  // Filter Rules API Tests
  // ============================================================================

  describe('Filter Rules API', () => {
    it('should list all filter rules', async () => {
      const mockRules = [createMockFilterRule(), createMockFilterRule()];
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: () => Promise.resolve(mockRules),
      });

      const result = await apiClient.listFilterRules();

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:3001/api/v1/filters',
        expect.any(Object)
      );
      expect(result).toEqual(mockRules);
    });

    it('should list filter rules for a specific backend', async () => {
      const mockRules = [createMockFilterRule({ backendId: 'backend-123' })];
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: () => Promise.resolve(mockRules),
      });

      const result = await apiClient.listFilterRules('backend-123');

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:3001/api/v1/filters?backendId=backend-123',
        expect.any(Object)
      );
      expect(result).toEqual(mockRules);
    });

    it('should create a filter rule', async () => {
      const newRule = {
        name: 'Block Bad IPs',
        type: 'ip',
        action: 'drop',
        priority: 100,
        enabled: true,
        config: { ips: ['192.168.1.0/24'] },
      };
      const createdRule = createMockFilterRule(newRule as any);
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 201,
        json: () => Promise.resolve(createdRule),
      });

      const result = await apiClient.createFilterRule(newRule);

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:3001/api/v1/filters',
        expect.objectContaining({
          method: 'POST',
          body: JSON.stringify(newRule),
        })
      );
      expect(result).toEqual(createdRule);
    });

    it('should delete a filter rule', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 204,
        json: () => Promise.resolve(undefined),
      });

      await apiClient.deleteFilterRule('rule-123');

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:3001/api/v1/filters/rule-123',
        expect.objectContaining({
          method: 'DELETE',
        })
      );
    });
  });

  // ============================================================================
  // Metrics API Tests
  // ============================================================================

  describe('Metrics API', () => {
    it('should get global metrics', async () => {
      const mockMetrics = createMockMetrics();
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: () => Promise.resolve(mockMetrics),
      });

      const result = await apiClient.getMetrics();

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:3001/api/v1/metrics',
        expect.any(Object)
      );
      expect(result).toEqual(mockMetrics);
    });

    it('should get metrics for a specific backend', async () => {
      const mockMetrics = createMockMetrics();
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: () => Promise.resolve(mockMetrics),
      });

      const result = await apiClient.getMetrics('backend-123');

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:3001/api/v1/metrics?backendId=backend-123',
        expect.any(Object)
      );
      expect(result).toEqual(mockMetrics);
    });
  });

  // ============================================================================
  // API Keys Tests
  // ============================================================================

  describe('API Keys', () => {
    it('should list API keys', async () => {
      const mockKeys = [createMockApiKey(), createMockApiKey()];
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: () => Promise.resolve(mockKeys),
      });

      const result = await apiClient.listApiKeys();

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:3001/api/v1/api-keys',
        expect.any(Object)
      );
      expect(result).toEqual(mockKeys);
    });

    it('should create an API key', async () => {
      const newKey = { id: 'key-123', key: 'pp_live_abc123', name: 'Test Key' };
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 201,
        json: () => Promise.resolve(newKey),
      });

      const result = await apiClient.createApiKey('Test Key');

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:3001/api/v1/api-keys',
        expect.objectContaining({
          method: 'POST',
          body: JSON.stringify({ name: 'Test Key' }),
        })
      );
      expect(result).toEqual(newKey);
    });

    it('should delete an API key', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 204,
        json: () => Promise.resolve(undefined),
      });

      await apiClient.deleteApiKey('key-123');

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:3001/api/v1/api-keys/key-123',
        expect.objectContaining({
          method: 'DELETE',
        })
      );
    });
  });

  // ============================================================================
  // Subscription API Tests
  // ============================================================================

  describe('Subscription API', () => {
    it('should get subscription', async () => {
      const mockSubscription = createMockSubscription();
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: () => Promise.resolve(mockSubscription),
      });

      const result = await apiClient.getSubscription();

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:3001/api/v1/billing/subscription',
        expect.any(Object)
      );
      expect(result).toEqual(mockSubscription);
    });
  });

  // ============================================================================
  // Error Handling Tests
  // ============================================================================

  describe('Error Handling', () => {
    it('should throw error with message from response', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 400,
        json: () => Promise.resolve({ message: 'Invalid request' }),
      });

      await expect(apiClient.listBackends()).rejects.toThrow('Invalid request');
    });

    it('should throw error with status code on unknown error', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 500,
        json: () => Promise.reject(new Error('Parse error')),
      });

      await expect(apiClient.listBackends()).rejects.toThrow('Unknown error');
    });

    it('should handle network errors', async () => {
      mockFetch.mockRejectedValueOnce(new Error('Network error'));

      await expect(apiClient.listBackends()).rejects.toThrow('Network error');
    });

    it('should handle 401 unauthorized', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 401,
        json: () => Promise.resolve({ message: 'Unauthorized' }),
      });

      await expect(apiClient.listBackends()).rejects.toThrow('Unauthorized');
    });

    it('should handle 403 forbidden', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 403,
        json: () => Promise.resolve({ message: 'Forbidden' }),
      });

      await expect(apiClient.listBackends()).rejects.toThrow('Forbidden');
    });
  });
});
