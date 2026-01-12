/**
 * Component tests for UI components
 */

import { describe, it, expect, vi } from 'vitest';
import React from 'react';
import {
  renderWithProviders,
  screen,
  fireEvent,
  waitFor,
  createMockBackend,
  createMockFilterRule,
  createMockMetrics,
  createMockUser,
  createMockSubscription,
} from './test-utils';

// ============================================================================
// Button Component Tests
// ============================================================================

describe('Button Component', () => {
  // Mock button component for testing
  const Button = ({
    children,
    onClick,
    disabled,
    variant = 'default',
    size = 'default',
  }: {
    children: React.ReactNode;
    onClick?: () => void;
    disabled?: boolean;
    variant?: 'default' | 'destructive' | 'outline' | 'ghost';
    size?: 'default' | 'sm' | 'lg' | 'icon';
  }) => (
    <button
      onClick={onClick}
      disabled={disabled}
      data-variant={variant}
      data-size={size}
      className="button"
    >
      {children}
    </button>
  );

  it('should render with text', () => {
    renderWithProviders(<Button>Click me</Button>);
    expect(screen.getByText('Click me')).toBeInTheDocument();
  });

  it('should call onClick when clicked', () => {
    const handleClick = vi.fn();
    renderWithProviders(<Button onClick={handleClick}>Click me</Button>);

    fireEvent.click(screen.getByText('Click me'));
    expect(handleClick).toHaveBeenCalledTimes(1);
  });

  it('should not call onClick when disabled', () => {
    const handleClick = vi.fn();
    renderWithProviders(
      <Button onClick={handleClick} disabled>
        Click me
      </Button>
    );

    fireEvent.click(screen.getByText('Click me'));
    expect(handleClick).not.toHaveBeenCalled();
  });

  it('should render with different variants', () => {
    const { rerender } = renderWithProviders(<Button variant="default">Default</Button>);
    expect(screen.getByText('Default')).toHaveAttribute('data-variant', 'default');

    rerender(<Button variant="destructive">Destructive</Button>);
    expect(screen.getByText('Destructive')).toHaveAttribute('data-variant', 'destructive');

    rerender(<Button variant="outline">Outline</Button>);
    expect(screen.getByText('Outline')).toHaveAttribute('data-variant', 'outline');
  });

  it('should render with different sizes', () => {
    const { rerender } = renderWithProviders(<Button size="sm">Small</Button>);
    expect(screen.getByText('Small')).toHaveAttribute('data-size', 'sm');

    rerender(<Button size="lg">Large</Button>);
    expect(screen.getByText('Large')).toHaveAttribute('data-size', 'lg');
  });
});

// ============================================================================
// Input Component Tests
// ============================================================================

describe('Input Component', () => {
  const Input = ({
    value,
    onChange,
    placeholder,
    type = 'text',
    disabled,
    error,
  }: {
    value?: string;
    onChange?: (e: React.ChangeEvent<HTMLInputElement>) => void;
    placeholder?: string;
    type?: string;
    disabled?: boolean;
    error?: string;
  }) => (
    <div>
      <input
        type={type}
        value={value}
        onChange={onChange}
        placeholder={placeholder}
        disabled={disabled}
        aria-invalid={!!error}
      />
      {error && <span role="alert">{error}</span>}
    </div>
  );

  it('should render with placeholder', () => {
    renderWithProviders(<Input placeholder="Enter text" />);
    expect(screen.getByPlaceholderText('Enter text')).toBeInTheDocument();
  });

  it('should call onChange when typing', () => {
    const handleChange = vi.fn();
    renderWithProviders(<Input onChange={handleChange} placeholder="Type here" />);

    fireEvent.change(screen.getByPlaceholderText('Type here'), {
      target: { value: 'Hello' },
    });
    expect(handleChange).toHaveBeenCalled();
  });

  it('should display error message', () => {
    renderWithProviders(<Input error="This field is required" />);
    expect(screen.getByRole('alert')).toHaveTextContent('This field is required');
  });

  it('should be disabled when disabled prop is true', () => {
    renderWithProviders(<Input disabled placeholder="Disabled" />);
    expect(screen.getByPlaceholderText('Disabled')).toBeDisabled();
  });
});

// ============================================================================
// Card Component Tests
// ============================================================================

describe('Card Component', () => {
  const Card = ({
    title,
    description,
    children,
    footer,
  }: {
    title?: string;
    description?: string;
    children?: React.ReactNode;
    footer?: React.ReactNode;
  }) => (
    <div className="card" role="region">
      {title && <h3 className="card-title">{title}</h3>}
      {description && <p className="card-description">{description}</p>}
      <div className="card-content">{children}</div>
      {footer && <div className="card-footer">{footer}</div>}
    </div>
  );

  it('should render with title and description', () => {
    renderWithProviders(
      <Card title="Test Card" description="This is a test card" />
    );
    expect(screen.getByText('Test Card')).toBeInTheDocument();
    expect(screen.getByText('This is a test card')).toBeInTheDocument();
  });

  it('should render children', () => {
    renderWithProviders(
      <Card>
        <p>Card content</p>
      </Card>
    );
    expect(screen.getByText('Card content')).toBeInTheDocument();
  });

  it('should render footer', () => {
    renderWithProviders(
      <Card footer={<button>Action</button>}>Content</Card>
    );
    expect(screen.getByText('Action')).toBeInTheDocument();
  });
});

// ============================================================================
// Badge Component Tests
// ============================================================================

describe('Badge Component', () => {
  const Badge = ({
    children,
    variant = 'default',
  }: {
    children: React.ReactNode;
    variant?: 'default' | 'success' | 'warning' | 'error';
  }) => (
    <span className={`badge badge-${variant}`} data-variant={variant}>
      {children}
    </span>
  );

  it('should render with text', () => {
    renderWithProviders(<Badge>Active</Badge>);
    expect(screen.getByText('Active')).toBeInTheDocument();
  });

  it('should render with different variants', () => {
    const { rerender } = renderWithProviders(<Badge variant="success">Success</Badge>);
    expect(screen.getByText('Success')).toHaveAttribute('data-variant', 'success');

    rerender(<Badge variant="error">Error</Badge>);
    expect(screen.getByText('Error')).toHaveAttribute('data-variant', 'error');
  });
});

// ============================================================================
// Backend Card Component Tests
// ============================================================================

describe('Backend Card Component', () => {
  const BackendCard = ({
    backend,
    onToggle,
    onDelete,
    onEdit,
  }: {
    backend: ReturnType<typeof createMockBackend>;
    onToggle?: (id: string, enabled: boolean) => void;
    onDelete?: (id: string) => void;
    onEdit?: (id: string) => void;
  }) => (
    <div className="backend-card" data-testid="backend-card">
      <h4>{backend.name}</h4>
      <p>{backend.address}:{backend.port}</p>
      <span data-testid="status">{backend.status}</span>
      <span data-testid="protocol">{backend.protocol}</span>
      <div className="stats">
        <span data-testid="requests">{backend.stats.requests}</span>
        <span data-testid="blocked">{backend.stats.blocked}</span>
      </div>
      <button
        onClick={() => onToggle?.(backend.id, !backend.enabled)}
        data-testid="toggle-button"
      >
        {backend.enabled ? 'Disable' : 'Enable'}
      </button>
      <button onClick={() => onEdit?.(backend.id)} data-testid="edit-button">
        Edit
      </button>
      <button onClick={() => onDelete?.(backend.id)} data-testid="delete-button">
        Delete
      </button>
    </div>
  );

  it('should render backend information', () => {
    const backend = createMockBackend({
      name: 'Game Server',
      address: '192.168.1.100',
      port: 25565,
      status: 'healthy',
    });

    renderWithProviders(<BackendCard backend={backend} />);

    expect(screen.getByText('Game Server')).toBeInTheDocument();
    expect(screen.getByText('192.168.1.100:25565')).toBeInTheDocument();
    expect(screen.getByTestId('status')).toHaveTextContent('healthy');
  });

  it('should call onToggle when toggle button is clicked', () => {
    const backend = createMockBackend({ enabled: true });
    const handleToggle = vi.fn();

    renderWithProviders(<BackendCard backend={backend} onToggle={handleToggle} />);

    fireEvent.click(screen.getByTestId('toggle-button'));
    expect(handleToggle).toHaveBeenCalledWith(backend.id, false);
  });

  it('should call onDelete when delete button is clicked', () => {
    const backend = createMockBackend();
    const handleDelete = vi.fn();

    renderWithProviders(<BackendCard backend={backend} onDelete={handleDelete} />);

    fireEvent.click(screen.getByTestId('delete-button'));
    expect(handleDelete).toHaveBeenCalledWith(backend.id);
  });

  it('should display stats', () => {
    const backend = createMockBackend({
      stats: {
        requests: 10000,
        blocked: 500,
        passed: 9500,
        latency: 15,
        bytesIn: 1000000,
        bytesOut: 2000000,
      },
    });

    renderWithProviders(<BackendCard backend={backend} />);

    expect(screen.getByTestId('requests')).toHaveTextContent('10000');
    expect(screen.getByTestId('blocked')).toHaveTextContent('500');
  });
});

// ============================================================================
// Filter Rule Card Component Tests
// ============================================================================

describe('Filter Rule Card Component', () => {
  const FilterRuleCard = ({
    rule,
    onToggle,
    onDelete,
  }: {
    rule: ReturnType<typeof createMockFilterRule>;
    onToggle?: (id: string, enabled: boolean) => void;
    onDelete?: (id: string) => void;
  }) => (
    <div className="filter-rule-card" data-testid="filter-rule-card">
      <h4>{rule.name}</h4>
      <p>{rule.description}</p>
      <span data-testid="type">{rule.type}</span>
      <span data-testid="action">{rule.action}</span>
      <span data-testid="priority">{rule.priority}</span>
      <span data-testid="matches">{rule.matches}</span>
      <button
        onClick={() => onToggle?.(rule.id, !rule.enabled)}
        data-testid="toggle-button"
      >
        {rule.enabled ? 'Disable' : 'Enable'}
      </button>
      <button onClick={() => onDelete?.(rule.id)} data-testid="delete-button">
        Delete
      </button>
    </div>
  );

  it('should render filter rule information', () => {
    const rule = createMockFilterRule({
      name: 'Block Bad IPs',
      type: 'ip',
      action: 'drop',
      priority: 100,
    });

    renderWithProviders(<FilterRuleCard rule={rule} />);

    expect(screen.getByText('Block Bad IPs')).toBeInTheDocument();
    expect(screen.getByTestId('type')).toHaveTextContent('ip');
    expect(screen.getByTestId('action')).toHaveTextContent('drop');
    expect(screen.getByTestId('priority')).toHaveTextContent('100');
  });

  it('should call onToggle when toggle button is clicked', () => {
    const rule = createMockFilterRule({ enabled: true });
    const handleToggle = vi.fn();

    renderWithProviders(<FilterRuleCard rule={rule} onToggle={handleToggle} />);

    fireEvent.click(screen.getByTestId('toggle-button'));
    expect(handleToggle).toHaveBeenCalledWith(rule.id, false);
  });

  it('should display match count', () => {
    const rule = createMockFilterRule({ matches: 12345 });

    renderWithProviders(<FilterRuleCard rule={rule} />);

    expect(screen.getByTestId('matches')).toHaveTextContent('12345');
  });
});

// ============================================================================
// Metrics Dashboard Component Tests
// ============================================================================

describe('Metrics Dashboard Component', () => {
  const MetricsDashboard = ({
    metrics,
  }: {
    metrics: ReturnType<typeof createMockMetrics>;
  }) => (
    <div className="metrics-dashboard" data-testid="metrics-dashboard">
      <div className="metric">
        <label>Total Requests</label>
        <span data-testid="total-requests">{metrics.totalRequests.toLocaleString()}</span>
      </div>
      <div className="metric">
        <label>Blocked Requests</label>
        <span data-testid="blocked-requests">{metrics.blockedRequests.toLocaleString()}</span>
      </div>
      <div className="metric">
        <label>Pass Rate</label>
        <span data-testid="pass-rate">
          {((metrics.passedRequests / metrics.totalRequests) * 100).toFixed(1)}%
        </span>
      </div>
      <div className="metric">
        <label>Avg Latency</label>
        <span data-testid="avg-latency">{metrics.avgLatency}ms</span>
      </div>
      <div className="metric">
        <label>Active Connections</label>
        <span data-testid="active-connections">{metrics.activeConnections}</span>
      </div>
    </div>
  );

  it('should render all metrics', () => {
    const metrics = createMockMetrics({
      totalRequests: 100000,
      blockedRequests: 5000,
      passedRequests: 95000,
      avgLatency: 15,
      activeConnections: 500,
    });

    renderWithProviders(<MetricsDashboard metrics={metrics} />);

    expect(screen.getByTestId('total-requests')).toHaveTextContent('100,000');
    expect(screen.getByTestId('blocked-requests')).toHaveTextContent('5,000');
    expect(screen.getByTestId('pass-rate')).toHaveTextContent('95.0%');
    expect(screen.getByTestId('avg-latency')).toHaveTextContent('15ms');
    expect(screen.getByTestId('active-connections')).toHaveTextContent('500');
  });
});

// ============================================================================
// Subscription Card Component Tests
// ============================================================================

describe('Subscription Card Component', () => {
  const SubscriptionCard = ({
    subscription,
    onUpgrade,
    onCancel,
  }: {
    subscription: ReturnType<typeof createMockSubscription>;
    onUpgrade?: () => void;
    onCancel?: () => void;
  }) => (
    <div className="subscription-card" data-testid="subscription-card">
      <h4>Current Plan: {subscription.plan}</h4>
      <span data-testid="status">{subscription.status}</span>
      <div className="usage">
        <span data-testid="backends">
          {subscription.usage.backends} / {subscription.limits.backends} backends
        </span>
        <span data-testid="requests">
          {subscription.usage.requestsThisMonth.toLocaleString()} requests
        </span>
      </div>
      <div className="actions">
        <button onClick={onUpgrade} data-testid="upgrade-button">
          Upgrade
        </button>
        <button onClick={onCancel} data-testid="cancel-button">
          Cancel
        </button>
      </div>
    </div>
  );

  it('should render subscription information', () => {
    const subscription = createMockSubscription({
      plan: 'pro',
      status: 'active',
      limits: { backends: 10, requestsPerMonth: 10000000, filterRules: 100, retentionDays: 90 },
      usage: { backends: 3, requestsThisMonth: 2500000, filterRules: 15 },
    });

    renderWithProviders(<SubscriptionCard subscription={subscription} />);

    expect(screen.getByText(/Current Plan: pro/i)).toBeInTheDocument();
    expect(screen.getByTestId('status')).toHaveTextContent('active');
    expect(screen.getByTestId('backends')).toHaveTextContent('3 / 10 backends');
  });

  it('should call onUpgrade when upgrade button is clicked', () => {
    const subscription = createMockSubscription();
    const handleUpgrade = vi.fn();

    renderWithProviders(<SubscriptionCard subscription={subscription} onUpgrade={handleUpgrade} />);

    fireEvent.click(screen.getByTestId('upgrade-button'));
    expect(handleUpgrade).toHaveBeenCalled();
  });

  it('should call onCancel when cancel button is clicked', () => {
    const subscription = createMockSubscription();
    const handleCancel = vi.fn();

    renderWithProviders(<SubscriptionCard subscription={subscription} onCancel={handleCancel} />);

    fireEvent.click(screen.getByTestId('cancel-button'));
    expect(handleCancel).toHaveBeenCalled();
  });
});

// ============================================================================
// Dialog Component Tests
// ============================================================================

describe('Dialog Component', () => {
  const Dialog = ({
    open,
    title,
    description,
    children,
    onClose,
    onConfirm,
  }: {
    open: boolean;
    title: string;
    description?: string;
    children?: React.ReactNode;
    onClose: () => void;
    onConfirm?: () => void;
  }) => {
    if (!open) return null;

    return (
      <div role="dialog" aria-labelledby="dialog-title" data-testid="dialog">
        <h2 id="dialog-title">{title}</h2>
        {description && <p>{description}</p>}
        <div>{children}</div>
        <div className="dialog-actions">
          <button onClick={onClose} data-testid="close-button">
            Close
          </button>
          {onConfirm && (
            <button onClick={onConfirm} data-testid="confirm-button">
              Confirm
            </button>
          )}
        </div>
      </div>
    );
  };

  it('should render when open', () => {
    renderWithProviders(
      <Dialog open={true} title="Test Dialog" onClose={() => {}} />
    );
    expect(screen.getByRole('dialog')).toBeInTheDocument();
    expect(screen.getByText('Test Dialog')).toBeInTheDocument();
  });

  it('should not render when closed', () => {
    renderWithProviders(
      <Dialog open={false} title="Test Dialog" onClose={() => {}} />
    );
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  });

  it('should call onClose when close button is clicked', () => {
    const handleClose = vi.fn();
    renderWithProviders(
      <Dialog open={true} title="Test Dialog" onClose={handleClose} />
    );

    fireEvent.click(screen.getByTestId('close-button'));
    expect(handleClose).toHaveBeenCalled();
  });

  it('should call onConfirm when confirm button is clicked', () => {
    const handleConfirm = vi.fn();
    renderWithProviders(
      <Dialog
        open={true}
        title="Test Dialog"
        onClose={() => {}}
        onConfirm={handleConfirm}
      />
    );

    fireEvent.click(screen.getByTestId('confirm-button'));
    expect(handleConfirm).toHaveBeenCalled();
  });
});
