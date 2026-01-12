import { useSyncExternalStore, useCallback } from "react"
import type { User } from "./api"
import { apiClient } from "./api"

// Auth state types
interface AuthState {
  user: User | null
  token: string | null
  isAuthenticated: boolean
  isLoading: boolean
}

// Storage keys
const TOKEN_KEY = "piston_auth_token"
const USER_KEY = "piston_auth_user"

// Initial state
const getInitialState = (): AuthState => {
  if (typeof window === "undefined") {
    return {
      user: null,
      token: null,
      isAuthenticated: false,
      isLoading: true,
    }
  }

  const token = localStorage.getItem(TOKEN_KEY)
  const userStr = localStorage.getItem(USER_KEY)
  const user = userStr ? (JSON.parse(userStr) as User) : null

  if (token) {
    apiClient.setToken(token)
  }

  return {
    user,
    token,
    isAuthenticated: !!token && !!user,
    isLoading: false,
  }
}

// Simple store implementation
let state = getInitialState()
const listeners = new Set<() => void>()

const getSnapshot = () => state
const getServerSnapshot = () => ({
  user: null,
  token: null,
  isAuthenticated: false,
  isLoading: true,
})

const subscribe = (listener: () => void) => {
  listeners.add(listener)
  return () => listeners.delete(listener)
}

const setState = (newState: Partial<AuthState>) => {
  state = { ...state, ...newState }
  listeners.forEach((listener) => listener())
}

// Auth actions
export const authActions = {
  async login(email: string, password: string) {
    setState({ isLoading: true })
    try {
      // In development, mock the login
      if (import.meta.env.DEV) {
        await new Promise((r) => setTimeout(r, 1000))
        const mockUser: User = {
          id: "1",
          email,
          name: email.split("@")[0],
          plan: "pro",
          createdAt: new Date().toISOString(),
        }
        const mockToken = "mock_token_" + Date.now()

        localStorage.setItem(TOKEN_KEY, mockToken)
        localStorage.setItem(USER_KEY, JSON.stringify(mockUser))
        apiClient.setToken(mockToken)

        setState({
          user: mockUser,
          token: mockToken,
          isAuthenticated: true,
          isLoading: false,
        })
        return { user: mockUser, token: mockToken }
      }

      const result = await apiClient.login(email, password)
      localStorage.setItem(TOKEN_KEY, result.token)
      localStorage.setItem(USER_KEY, JSON.stringify(result.user))
      apiClient.setToken(result.token)

      setState({
        user: result.user,
        token: result.token,
        isAuthenticated: true,
        isLoading: false,
      })
      return result
    } catch (error) {
      setState({ isLoading: false })
      throw error
    }
  },

  async register(email: string, password: string, name: string) {
    setState({ isLoading: true })
    try {
      // In development, mock the registration
      if (import.meta.env.DEV) {
        await new Promise((r) => setTimeout(r, 1000))
        const mockUser: User = {
          id: "1",
          email,
          name,
          plan: "free",
          createdAt: new Date().toISOString(),
        }
        const mockToken = "mock_token_" + Date.now()

        localStorage.setItem(TOKEN_KEY, mockToken)
        localStorage.setItem(USER_KEY, JSON.stringify(mockUser))
        apiClient.setToken(mockToken)

        setState({
          user: mockUser,
          token: mockToken,
          isAuthenticated: true,
          isLoading: false,
        })
        return { user: mockUser, token: mockToken }
      }

      const result = await apiClient.register(email, password, name)
      localStorage.setItem(TOKEN_KEY, result.token)
      localStorage.setItem(USER_KEY, JSON.stringify(result.user))
      apiClient.setToken(result.token)

      setState({
        user: result.user,
        token: result.token,
        isAuthenticated: true,
        isLoading: false,
      })
      return result
    } catch (error) {
      setState({ isLoading: false })
      throw error
    }
  },

  async logout() {
    try {
      if (!import.meta.env.DEV) {
        await apiClient.logout()
      }
    } finally {
      localStorage.removeItem(TOKEN_KEY)
      localStorage.removeItem(USER_KEY)
      apiClient.setToken(null)

      setState({
        user: null,
        token: null,
        isAuthenticated: false,
        isLoading: false,
      })
    }
  },

  updateUser(user: User) {
    localStorage.setItem(USER_KEY, JSON.stringify(user))
    setState({ user })
  },

  checkAuth() {
    const token = localStorage.getItem(TOKEN_KEY)
    const userStr = localStorage.getItem(USER_KEY)

    if (!token || !userStr) {
      setState({
        user: null,
        token: null,
        isAuthenticated: false,
        isLoading: false,
      })
      return false
    }

    apiClient.setToken(token)
    setState({
      user: JSON.parse(userStr),
      token,
      isAuthenticated: true,
      isLoading: false,
    })
    return true
  },
}

// React hook for auth state
export function useAuth() {
  const authState = useSyncExternalStore(subscribe, getSnapshot, getServerSnapshot)

  const login = useCallback(async (email: string, password: string) => {
    return authActions.login(email, password)
  }, [])

  const register = useCallback(
    async (email: string, password: string, name: string) => {
      return authActions.register(email, password, name)
    },
    []
  )

  const logout = useCallback(async () => {
    return authActions.logout()
  }, [])

  return {
    ...authState,
    login,
    register,
    logout,
  }
}

// Export for use in route guards
export const getAuthState = () => state
