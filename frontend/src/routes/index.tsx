import { createFileRoute, Navigate } from "@tanstack/react-router"
import { getAuthState } from "@/lib/auth"

export const Route = createFileRoute("/")({
  component: RootRedirect,
})

function RootRedirect() {
  const authState = getAuthState()

  // If not authenticated, redirect to login
  if (!authState.isAuthenticated) {
    return <Navigate to="/login" />
  }

  // If authenticated, redirect to dashboard overview
  return <Navigate to="/dashboard" />
}
