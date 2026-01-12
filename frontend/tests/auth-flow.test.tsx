/**
 * Authentication flow tests
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import React, { useState } from 'react';
import {
  renderWithProviders,
  screen,
  fireEvent,
  waitFor,
  createMockUser,
} from './test-utils';

// ============================================================================
// Mock Auth Context and Hooks
// ============================================================================

interface AuthState {
  user: ReturnType<typeof createMockUser> | null;
  isLoading: boolean;
  isAuthenticated: boolean;
  error: string | null;
}

interface AuthContextValue extends AuthState {
  login: (email: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
  register: (email: string, password: string, name: string) => Promise<void>;
  resetPassword: (email: string) => Promise<void>;
  updateProfile: (data: { name?: string; email?: string }) => Promise<void>;
}

const createMockAuthContext = (overrides: Partial<AuthContextValue> = {}): AuthContextValue => ({
  user: null,
  isLoading: false,
  isAuthenticated: false,
  error: null,
  login: vi.fn(),
  logout: vi.fn(),
  register: vi.fn(),
  resetPassword: vi.fn(),
  updateProfile: vi.fn(),
  ...overrides,
});

// ============================================================================
// Login Form Component
// ============================================================================

interface LoginFormProps {
  onSubmit: (email: string, password: string) => Promise<void>;
  onForgotPassword?: () => void;
  onSignUp?: () => void;
  error?: string | null;
  isLoading?: boolean;
}

const LoginForm = ({
  onSubmit,
  onForgotPassword,
  onSignUp,
  error,
  isLoading,
}: LoginFormProps) => {
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [validationError, setValidationError] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setValidationError(null);

    if (!email) {
      setValidationError('Email is required');
      return;
    }
    if (!email.includes('@')) {
      setValidationError('Invalid email format');
      return;
    }
    if (!password) {
      setValidationError('Password is required');
      return;
    }
    if (password.length < 8) {
      setValidationError('Password must be at least 8 characters');
      return;
    }

    await onSubmit(email, password);
  };

  return (
    <form onSubmit={handleSubmit} data-testid="login-form">
      <h2>Sign In</h2>

      {(error || validationError) && (
        <div role="alert" data-testid="error-message">
          {error || validationError}
        </div>
      )}

      <div>
        <label htmlFor="email">Email</label>
        <input
          id="email"
          type="email"
          value={email}
          onChange={(e) => setEmail(e.target.value)}
          placeholder="Enter your email"
          disabled={isLoading}
        />
      </div>

      <div>
        <label htmlFor="password">Password</label>
        <input
          id="password"
          type="password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          placeholder="Enter your password"
          disabled={isLoading}
        />
      </div>

      <button type="submit" disabled={isLoading} data-testid="login-button">
        {isLoading ? 'Signing in...' : 'Sign In'}
      </button>

      {onForgotPassword && (
        <button type="button" onClick={onForgotPassword} data-testid="forgot-password">
          Forgot password?
        </button>
      )}

      {onSignUp && (
        <button type="button" onClick={onSignUp} data-testid="sign-up-link">
          Create account
        </button>
      )}
    </form>
  );
};

// ============================================================================
// Register Form Component
// ============================================================================

interface RegisterFormProps {
  onSubmit: (email: string, password: string, name: string) => Promise<void>;
  onSignIn?: () => void;
  error?: string | null;
  isLoading?: boolean;
}

const RegisterForm = ({
  onSubmit,
  onSignIn,
  error,
  isLoading,
}: RegisterFormProps) => {
  const [name, setName] = useState('');
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [validationError, setValidationError] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setValidationError(null);

    if (!name) {
      setValidationError('Name is required');
      return;
    }
    if (!email || !email.includes('@')) {
      setValidationError('Valid email is required');
      return;
    }
    if (password.length < 8) {
      setValidationError('Password must be at least 8 characters');
      return;
    }
    if (password !== confirmPassword) {
      setValidationError('Passwords do not match');
      return;
    }

    await onSubmit(email, password, name);
  };

  return (
    <form onSubmit={handleSubmit} data-testid="register-form">
      <h2>Create Account</h2>

      {(error || validationError) && (
        <div role="alert" data-testid="error-message">
          {error || validationError}
        </div>
      )}

      <div>
        <label htmlFor="name">Name</label>
        <input
          id="name"
          type="text"
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="Enter your name"
          disabled={isLoading}
        />
      </div>

      <div>
        <label htmlFor="register-email">Email</label>
        <input
          id="register-email"
          type="email"
          value={email}
          onChange={(e) => setEmail(e.target.value)}
          placeholder="Enter your email"
          disabled={isLoading}
        />
      </div>

      <div>
        <label htmlFor="register-password">Password</label>
        <input
          id="register-password"
          type="password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          placeholder="Create a password"
          disabled={isLoading}
        />
      </div>

      <div>
        <label htmlFor="confirm-password">Confirm Password</label>
        <input
          id="confirm-password"
          type="password"
          value={confirmPassword}
          onChange={(e) => setConfirmPassword(e.target.value)}
          placeholder="Confirm your password"
          disabled={isLoading}
        />
      </div>

      <button type="submit" disabled={isLoading} data-testid="register-button">
        {isLoading ? 'Creating account...' : 'Create Account'}
      </button>

      {onSignIn && (
        <button type="button" onClick={onSignIn} data-testid="sign-in-link">
          Already have an account?
        </button>
      )}
    </form>
  );
};

// ============================================================================
// Login Form Tests
// ============================================================================

describe('Login Form', () => {
  it('should render login form', () => {
    renderWithProviders(<LoginForm onSubmit={vi.fn()} />);

    expect(screen.getByText('Sign In')).toBeInTheDocument();
    expect(screen.getByLabelText('Email')).toBeInTheDocument();
    expect(screen.getByLabelText('Password')).toBeInTheDocument();
    expect(screen.getByTestId('login-button')).toBeInTheDocument();
  });

  it('should call onSubmit with email and password', async () => {
    const handleSubmit = vi.fn().mockResolvedValue(undefined);
    renderWithProviders(<LoginForm onSubmit={handleSubmit} />);

    fireEvent.change(screen.getByLabelText('Email'), {
      target: { value: 'test@example.com' },
    });
    fireEvent.change(screen.getByLabelText('Password'), {
      target: { value: 'password123' },
    });
    fireEvent.click(screen.getByTestId('login-button'));

    await waitFor(() => {
      expect(handleSubmit).toHaveBeenCalledWith('test@example.com', 'password123');
    });
  });

  it('should show validation error for empty email', async () => {
    renderWithProviders(<LoginForm onSubmit={vi.fn()} />);

    fireEvent.change(screen.getByLabelText('Password'), {
      target: { value: 'password123' },
    });
    fireEvent.click(screen.getByTestId('login-button'));

    await waitFor(() => {
      expect(screen.getByTestId('error-message')).toHaveTextContent('Email is required');
    });
  });

  it('should show validation error for invalid email', async () => {
    renderWithProviders(<LoginForm onSubmit={vi.fn()} />);

    fireEvent.change(screen.getByLabelText('Email'), {
      target: { value: 'invalid-email' },
    });
    fireEvent.change(screen.getByLabelText('Password'), {
      target: { value: 'password123' },
    });
    fireEvent.click(screen.getByTestId('login-button'));

    await waitFor(() => {
      expect(screen.getByTestId('error-message')).toHaveTextContent('Invalid email format');
    });
  });

  it('should show validation error for short password', async () => {
    renderWithProviders(<LoginForm onSubmit={vi.fn()} />);

    fireEvent.change(screen.getByLabelText('Email'), {
      target: { value: 'test@example.com' },
    });
    fireEvent.change(screen.getByLabelText('Password'), {
      target: { value: 'short' },
    });
    fireEvent.click(screen.getByTestId('login-button'));

    await waitFor(() => {
      expect(screen.getByTestId('error-message')).toHaveTextContent(
        'Password must be at least 8 characters'
      );
    });
  });

  it('should display server error', () => {
    renderWithProviders(<LoginForm onSubmit={vi.fn()} error="Invalid credentials" />);

    expect(screen.getByTestId('error-message')).toHaveTextContent('Invalid credentials');
  });

  it('should disable form while loading', () => {
    renderWithProviders(<LoginForm onSubmit={vi.fn()} isLoading={true} />);

    expect(screen.getByLabelText('Email')).toBeDisabled();
    expect(screen.getByLabelText('Password')).toBeDisabled();
    expect(screen.getByTestId('login-button')).toBeDisabled();
    expect(screen.getByTestId('login-button')).toHaveTextContent('Signing in...');
  });

  it('should call onForgotPassword when clicked', () => {
    const handleForgotPassword = vi.fn();
    renderWithProviders(
      <LoginForm onSubmit={vi.fn()} onForgotPassword={handleForgotPassword} />
    );

    fireEvent.click(screen.getByTestId('forgot-password'));
    expect(handleForgotPassword).toHaveBeenCalled();
  });

  it('should call onSignUp when clicked', () => {
    const handleSignUp = vi.fn();
    renderWithProviders(<LoginForm onSubmit={vi.fn()} onSignUp={handleSignUp} />);

    fireEvent.click(screen.getByTestId('sign-up-link'));
    expect(handleSignUp).toHaveBeenCalled();
  });
});

// ============================================================================
// Register Form Tests
// ============================================================================

describe('Register Form', () => {
  it('should render register form', () => {
    renderWithProviders(<RegisterForm onSubmit={vi.fn()} />);

    expect(screen.getByText('Create Account')).toBeInTheDocument();
    expect(screen.getByLabelText('Name')).toBeInTheDocument();
    expect(screen.getByLabelText('Email')).toBeInTheDocument();
    expect(screen.getByLabelText('Password')).toBeInTheDocument();
    expect(screen.getByLabelText('Confirm Password')).toBeInTheDocument();
    expect(screen.getByTestId('register-button')).toBeInTheDocument();
  });

  it('should call onSubmit with valid data', async () => {
    const handleSubmit = vi.fn().mockResolvedValue(undefined);
    renderWithProviders(<RegisterForm onSubmit={handleSubmit} />);

    fireEvent.change(screen.getByLabelText('Name'), {
      target: { value: 'John Doe' },
    });
    fireEvent.change(screen.getByLabelText('Email'), {
      target: { value: 'john@example.com' },
    });
    fireEvent.change(screen.getByLabelText('Password'), {
      target: { value: 'password123' },
    });
    fireEvent.change(screen.getByLabelText('Confirm Password'), {
      target: { value: 'password123' },
    });
    fireEvent.click(screen.getByTestId('register-button'));

    await waitFor(() => {
      expect(handleSubmit).toHaveBeenCalledWith('john@example.com', 'password123', 'John Doe');
    });
  });

  it('should show error for empty name', async () => {
    renderWithProviders(<RegisterForm onSubmit={vi.fn()} />);

    fireEvent.change(screen.getByLabelText('Email'), {
      target: { value: 'john@example.com' },
    });
    fireEvent.change(screen.getByLabelText('Password'), {
      target: { value: 'password123' },
    });
    fireEvent.change(screen.getByLabelText('Confirm Password'), {
      target: { value: 'password123' },
    });
    fireEvent.click(screen.getByTestId('register-button'));

    await waitFor(() => {
      expect(screen.getByTestId('error-message')).toHaveTextContent('Name is required');
    });
  });

  it('should show error for password mismatch', async () => {
    renderWithProviders(<RegisterForm onSubmit={vi.fn()} />);

    fireEvent.change(screen.getByLabelText('Name'), {
      target: { value: 'John Doe' },
    });
    fireEvent.change(screen.getByLabelText('Email'), {
      target: { value: 'john@example.com' },
    });
    fireEvent.change(screen.getByLabelText('Password'), {
      target: { value: 'password123' },
    });
    fireEvent.change(screen.getByLabelText('Confirm Password'), {
      target: { value: 'different123' },
    });
    fireEvent.click(screen.getByTestId('register-button'));

    await waitFor(() => {
      expect(screen.getByTestId('error-message')).toHaveTextContent('Passwords do not match');
    });
  });

  it('should disable form while loading', () => {
    renderWithProviders(<RegisterForm onSubmit={vi.fn()} isLoading={true} />);

    expect(screen.getByLabelText('Name')).toBeDisabled();
    expect(screen.getByLabelText('Email')).toBeDisabled();
    expect(screen.getByLabelText('Password')).toBeDisabled();
    expect(screen.getByLabelText('Confirm Password')).toBeDisabled();
    expect(screen.getByTestId('register-button')).toBeDisabled();
  });

  it('should call onSignIn when clicked', () => {
    const handleSignIn = vi.fn();
    renderWithProviders(<RegisterForm onSubmit={vi.fn()} onSignIn={handleSignIn} />);

    fireEvent.click(screen.getByTestId('sign-in-link'));
    expect(handleSignIn).toHaveBeenCalled();
  });
});

// ============================================================================
// Auth State Tests
// ============================================================================

describe('Auth State', () => {
  const AuthStatus = ({ auth }: { auth: AuthContextValue }) => (
    <div>
      {auth.isLoading && <span data-testid="loading">Loading...</span>}
      {auth.isAuthenticated && auth.user && (
        <div data-testid="authenticated">
          <span data-testid="user-email">{auth.user.email}</span>
          <button onClick={() => auth.logout()} data-testid="logout-button">
            Logout
          </button>
        </div>
      )}
      {!auth.isAuthenticated && !auth.isLoading && (
        <span data-testid="unauthenticated">Not logged in</span>
      )}
    </div>
  );

  it('should show loading state', () => {
    const auth = createMockAuthContext({ isLoading: true });
    renderWithProviders(<AuthStatus auth={auth} />);

    expect(screen.getByTestId('loading')).toBeInTheDocument();
  });

  it('should show unauthenticated state', () => {
    const auth = createMockAuthContext({ isAuthenticated: false });
    renderWithProviders(<AuthStatus auth={auth} />);

    expect(screen.getByTestId('unauthenticated')).toBeInTheDocument();
  });

  it('should show authenticated state with user', () => {
    const user = createMockUser({ email: 'test@example.com' });
    const auth = createMockAuthContext({
      isAuthenticated: true,
      user,
    });
    renderWithProviders(<AuthStatus auth={auth} />);

    expect(screen.getByTestId('authenticated')).toBeInTheDocument();
    expect(screen.getByTestId('user-email')).toHaveTextContent('test@example.com');
  });

  it('should call logout when logout button is clicked', () => {
    const logoutFn = vi.fn();
    const user = createMockUser();
    const auth = createMockAuthContext({
      isAuthenticated: true,
      user,
      logout: logoutFn,
    });
    renderWithProviders(<AuthStatus auth={auth} />);

    fireEvent.click(screen.getByTestId('logout-button'));
    expect(logoutFn).toHaveBeenCalled();
  });
});

// ============================================================================
// Protected Route Tests
// ============================================================================

describe('Protected Route', () => {
  interface ProtectedRouteProps {
    auth: AuthContextValue;
    children: React.ReactNode;
    fallback?: React.ReactNode;
  }

  const ProtectedRoute = ({ auth, children, fallback }: ProtectedRouteProps) => {
    if (auth.isLoading) {
      return <div data-testid="loading">Loading...</div>;
    }

    if (!auth.isAuthenticated) {
      return <>{fallback || <div data-testid="redirect">Redirecting to login...</div>}</>;
    }

    return <>{children}</>;
  };

  it('should show loading while auth is loading', () => {
    const auth = createMockAuthContext({ isLoading: true });
    renderWithProviders(
      <ProtectedRoute auth={auth}>
        <div>Protected Content</div>
      </ProtectedRoute>
    );

    expect(screen.getByTestId('loading')).toBeInTheDocument();
    expect(screen.queryByText('Protected Content')).not.toBeInTheDocument();
  });

  it('should show protected content when authenticated', () => {
    const auth = createMockAuthContext({
      isAuthenticated: true,
      user: createMockUser(),
    });
    renderWithProviders(
      <ProtectedRoute auth={auth}>
        <div>Protected Content</div>
      </ProtectedRoute>
    );

    expect(screen.getByText('Protected Content')).toBeInTheDocument();
  });

  it('should redirect when not authenticated', () => {
    const auth = createMockAuthContext({ isAuthenticated: false });
    renderWithProviders(
      <ProtectedRoute auth={auth}>
        <div>Protected Content</div>
      </ProtectedRoute>
    );

    expect(screen.getByTestId('redirect')).toBeInTheDocument();
    expect(screen.queryByText('Protected Content')).not.toBeInTheDocument();
  });

  it('should show custom fallback when not authenticated', () => {
    const auth = createMockAuthContext({ isAuthenticated: false });
    renderWithProviders(
      <ProtectedRoute auth={auth} fallback={<div>Please login</div>}>
        <div>Protected Content</div>
      </ProtectedRoute>
    );

    expect(screen.getByText('Please login')).toBeInTheDocument();
  });
});
