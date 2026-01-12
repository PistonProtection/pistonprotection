// Simple auth client for PistonProtection
// In production, this would integrate with better-auth

import { useState } from 'react'

export interface User {
  id: string
  name: string
  email: string
  image?: string
  emailVerified: boolean
  role?: string
  createdAt: string
}

export interface Session {
  user: User | null
  isPending: boolean
  error: Error | null
}

export interface AuthError {
  message: string
}

export interface AuthResult {
  error?: AuthError
}

// Mock auth state - in production, use better-auth
const mockUser: User = {
  id: 'user_1',
  name: 'John Doe',
  email: 'john@example.com',
  emailVerified: true,
  createdAt: new Date().toISOString(),
}

let currentSession: Session = {
  user: mockUser,
  isPending: false,
  error: null,
}

// Hook to get current session
export function useSession(): Session & { data: Session } {
  const [session] = useState<Session>(currentSession)
  return {
    ...session,
    data: session,
  }
}

// Email sign in
export const signIn = {
  email: async (options: {
    email: string
    password: string
    callbackURL?: string
    rememberMe?: boolean
  }): Promise<AuthResult> => {
    console.log('Sign in with email:', options.email)
    currentSession = {
      user: mockUser,
      isPending: false,
      error: null,
    }
    if (options.callbackURL) {
      window.location.href = options.callbackURL
    }
    return {}
  },
  social: async (options: {
    provider: 'github' | 'google' | 'discord'
    callbackURL?: string
  }): Promise<void> => {
    console.log('Social sign in:', options.provider)
    const callbackParam = options.callbackURL ? `?callbackURL=${encodeURIComponent(options.callbackURL)}` : ''
    window.location.href = `/api/auth/${options.provider}${callbackParam}`
  }
}

// Sign up function
export const signUp = {
  email: async (options: {
    name: string
    email: string
    password: string
    callbackURL?: string
  }): Promise<AuthResult> => {
    console.log('Sign up:', options)
    currentSession = {
      user: { ...mockUser, name: options.name, email: options.email },
      isPending: false,
      error: null,
    }
    if (options.callbackURL) {
      window.location.href = options.callbackURL
    }
    return {}
  },
  social: async (options: {
    provider: 'github' | 'google' | 'discord'
    callbackURL?: string
  }): Promise<void> => {
    console.log('Social sign up:', options.provider)
    const callbackParam = options.callbackURL ? `?callbackURL=${encodeURIComponent(options.callbackURL)}` : ''
    window.location.href = `/api/auth/${options.provider}${callbackParam}`
  }
}

// Sign out function
export async function signOut(): Promise<void> {
  console.log('Sign out')
  currentSession = { user: null, isPending: false, error: null }
}

// Forget password function
export async function forgetPassword(options: {
  email: string
  redirectTo?: string
}): Promise<AuthResult> {
  console.log('Password reset:', options.email)
  return {}
}

// Auth client singleton (for compatibility)
export const authClient = {
  useSession,
  signIn,
  signUp,
  signOut,
  forgetPassword,
}
