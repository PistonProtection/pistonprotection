import { useState } from "react"
import { createFileRoute } from "@tanstack/react-router"
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query"
import {
  Activity,
  MoreHorizontal,
  Pencil,
  Plus,
  RefreshCw,
  Server,
  Trash2,
} from "lucide-react"
import { toast } from "sonner"

import { cn } from "@/lib/utils"
import { backendsOptions, apiClient, type Backend } from "@/lib/api"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { Switch } from "@/components/ui/switch"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { Skeleton } from "@/components/ui/skeleton"

export const Route = createFileRoute("/dashboard/backends")({
  component: BackendsPage,
})

function BackendsPage() {
  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false)
  const [editingBackend, setEditingBackend] = useState<Backend | null>(null)
  const [deleteConfirmBackend, setDeleteConfirmBackend] = useState<Backend | null>(
    null
  )

  const queryClient = useQueryClient()
  const { data: backends, isLoading, refetch } = useQuery(backendsOptions)

  const deleteMutation = useMutation({
    mutationFn: (id: string) => {
      // In dev mode, just simulate the deletion
      if (import.meta.env.DEV) {
        return new Promise<void>((resolve) => setTimeout(resolve, 500))
      }
      return apiClient.deleteBackend(id)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["backends"] })
      toast.success("Backend deleted successfully")
      setDeleteConfirmBackend(null)
    },
    onError: () => {
      toast.error("Failed to delete backend")
    },
  })

  const healthyCount = backends?.filter((b) => b.status === "healthy").length || 0
  const totalCount = backends?.length || 0

  return (
    <div className="flex-1 space-y-6 p-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Backends</h1>
          <p className="text-muted-foreground">
            Manage your backend servers and their health checks.
          </p>
        </div>
        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => refetch()}
            disabled={isLoading}
          >
            <RefreshCw
              className={cn("mr-2 size-4", isLoading && "animate-spin")}
            />
            Refresh
          </Button>
          <Dialog open={isCreateDialogOpen} onOpenChange={setIsCreateDialogOpen}>
            <DialogTrigger asChild>
              <Button size="sm">
                <Plus className="mr-2 size-4" />
                Add Backend
              </Button>
            </DialogTrigger>
            <DialogContent className="sm:max-w-md">
              <BackendForm
                onSuccess={() => {
                  setIsCreateDialogOpen(false)
                  queryClient.invalidateQueries({ queryKey: ["backends"] })
                }}
                onCancel={() => setIsCreateDialogOpen(false)}
              />
            </DialogContent>
          </Dialog>
        </div>
      </div>

      {/* Stats Cards */}
      <div className="grid gap-4 md:grid-cols-3">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardDescription className="text-sm font-medium">
              Total Backends
            </CardDescription>
            <Server className="size-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{totalCount}</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardDescription className="text-sm font-medium">Healthy</CardDescription>
            <Activity className="size-4 text-green-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-green-600 dark:text-green-400">
              {healthyCount}
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardDescription className="text-sm font-medium">Unhealthy</CardDescription>
            <Activity className="size-4 text-red-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-red-600 dark:text-red-400">
              {totalCount - healthyCount}
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Backends Table */}
      <Card>
        <CardHeader>
          <CardTitle>Backend Servers</CardTitle>
          <CardDescription>
            A list of all your configured backend servers.
          </CardDescription>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="space-y-4">
              {[1, 2, 3].map((i) => (
                <div key={i} className="flex items-center gap-4">
                  <Skeleton className="size-4" />
                  <Skeleton className="h-4 w-40" />
                  <Skeleton className="h-4 w-32" />
                  <Skeleton className="h-4 w-20" />
                  <Skeleton className="h-5 w-16 ml-auto" />
                </div>
              ))}
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Name</TableHead>
                  <TableHead>Host</TableHead>
                  <TableHead>Protocol</TableHead>
                  <TableHead>Health Check</TableHead>
                  <TableHead>RPS</TableHead>
                  <TableHead>Latency</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead className="w-12"></TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {backends?.map((backend) => (
                  <TableRow key={backend.id}>
                    <TableCell className="font-medium">{backend.name}</TableCell>
                    <TableCell>
                      {backend.host}:{backend.port}
                    </TableCell>
                    <TableCell>
                      <Badge variant="outline" className="uppercase">
                        {backend.protocol}
                      </Badge>
                    </TableCell>
                    <TableCell>
                      {backend.healthCheck.enabled ? (
                        <span className="text-green-600 dark:text-green-400">
                          {backend.healthCheck.path} ({backend.healthCheck.interval}
                          s)
                        </span>
                      ) : (
                        <span className="text-muted-foreground">Disabled</span>
                      )}
                    </TableCell>
                    <TableCell>
                      {backend.requestsPerSecond.toLocaleString()}
                    </TableCell>
                    <TableCell>{backend.latency}ms</TableCell>
                    <TableCell>
                      <Badge
                        variant={
                          backend.status === "healthy"
                            ? "default"
                            : backend.status === "unhealthy"
                              ? "destructive"
                              : "secondary"
                        }
                        className={
                          backend.status === "healthy"
                            ? "bg-green-600"
                            : undefined
                        }
                      >
                        {backend.status}
                      </Badge>
                    </TableCell>
                    <TableCell>
                      <DropdownMenu>
                        <DropdownMenuTrigger asChild>
                          <Button variant="ghost" size="icon-sm">
                            <MoreHorizontal className="size-4" />
                            <span className="sr-only">Actions</span>
                          </Button>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent align="end">
                          <DropdownMenuItem
                            onClick={() => setEditingBackend(backend)}
                          >
                            <Pencil className="mr-2 size-4" />
                            Edit
                          </DropdownMenuItem>
                          <DropdownMenuSeparator />
                          <DropdownMenuItem
                            onClick={() => setDeleteConfirmBackend(backend)}
                            className="text-destructive focus:text-destructive"
                          >
                            <Trash2 className="mr-2 size-4" />
                            Delete
                          </DropdownMenuItem>
                        </DropdownMenuContent>
                      </DropdownMenu>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>

      {/* Edit Dialog */}
      <Dialog
        open={!!editingBackend}
        onOpenChange={() => setEditingBackend(null)}
      >
        <DialogContent className="sm:max-w-md">
          {editingBackend && (
            <BackendForm
              backend={editingBackend}
              onSuccess={() => {
                setEditingBackend(null)
                queryClient.invalidateQueries({ queryKey: ["backends"] })
              }}
              onCancel={() => setEditingBackend(null)}
            />
          )}
        </DialogContent>
      </Dialog>

      {/* Delete Confirmation Dialog */}
      <Dialog
        open={!!deleteConfirmBackend}
        onOpenChange={() => setDeleteConfirmBackend(null)}
      >
        <DialogContent className="sm:max-w-md" showCloseButton={false}>
          <DialogHeader>
            <DialogTitle>Delete Backend</DialogTitle>
            <DialogDescription>
              Are you sure you want to delete{" "}
              <strong>{deleteConfirmBackend?.name}</strong>? This action cannot
              be undone.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => setDeleteConfirmBackend(null)}
            >
              Cancel
            </Button>
            <Button
              variant="destructive"
              onClick={() =>
                deleteConfirmBackend &&
                deleteMutation.mutate(deleteConfirmBackend.id)
              }
              disabled={deleteMutation.isPending}
            >
              {deleteMutation.isPending ? "Deleting..." : "Delete"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}

function BackendForm({
  backend,
  onSuccess,
  onCancel,
}: {
  backend?: Backend
  onSuccess: () => void
  onCancel: () => void
}) {
  const [formData, setFormData] = useState({
    name: backend?.name || "",
    host: backend?.host || "",
    port: backend?.port?.toString() || "443",
    protocol: backend?.protocol || "https",
    healthCheckEnabled: backend?.healthCheck.enabled ?? true,
    healthCheckPath: backend?.healthCheck.path || "/health",
    healthCheckInterval: backend?.healthCheck.interval?.toString() || "30",
  })
  const [isSubmitting, setIsSubmitting] = useState(false)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setIsSubmitting(true)

    try {
      // Simulate API call in dev mode
      await new Promise((resolve) => setTimeout(resolve, 1000))
      toast.success(
        backend ? "Backend updated successfully" : "Backend created successfully"
      )
      onSuccess()
    } catch {
      toast.error(backend ? "Failed to update backend" : "Failed to create backend")
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <form onSubmit={handleSubmit}>
      <DialogHeader>
        <DialogTitle>{backend ? "Edit Backend" : "Add Backend"}</DialogTitle>
        <DialogDescription>
          {backend
            ? "Update your backend server configuration."
            : "Configure a new backend server."}
        </DialogDescription>
      </DialogHeader>

      <div className="grid gap-4 py-4">
        <div className="grid gap-2">
          <Label htmlFor="name">Name</Label>
          <Input
            id="name"
            value={formData.name}
            onChange={(e) =>
              setFormData({ ...formData, name: e.target.value })
            }
            placeholder="Primary API Server"
            required
          />
        </div>

        <div className="grid grid-cols-2 gap-4">
          <div className="grid gap-2">
            <Label htmlFor="host">Host</Label>
            <Input
              id="host"
              value={formData.host}
              onChange={(e) =>
                setFormData({ ...formData, host: e.target.value })
              }
              placeholder="api.example.com"
              required
            />
          </div>
          <div className="grid gap-2">
            <Label htmlFor="port">Port</Label>
            <Input
              id="port"
              type="number"
              value={formData.port}
              onChange={(e) =>
                setFormData({ ...formData, port: e.target.value })
              }
              placeholder="443"
              required
            />
          </div>
        </div>

        <div className="grid gap-2">
          <Label htmlFor="protocol">Protocol</Label>
          <Select
            value={formData.protocol}
            onValueChange={(value) =>
              setFormData({ ...formData, protocol: value })
            }
          >
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="https">HTTPS</SelectItem>
              <SelectItem value="http">HTTP</SelectItem>
              <SelectItem value="tcp">TCP</SelectItem>
              <SelectItem value="udp">UDP</SelectItem>
            </SelectContent>
          </Select>
        </div>

        <div className="flex items-center justify-between">
          <Label htmlFor="healthCheck">Enable Health Check</Label>
          <Switch
            id="healthCheck"
            checked={formData.healthCheckEnabled}
            onCheckedChange={(checked) =>
              setFormData({ ...formData, healthCheckEnabled: checked })
            }
          />
        </div>

        {formData.healthCheckEnabled && (
          <div className="grid grid-cols-2 gap-4">
            <div className="grid gap-2">
              <Label htmlFor="healthPath">Health Check Path</Label>
              <Input
                id="healthPath"
                value={formData.healthCheckPath}
                onChange={(e) =>
                  setFormData({ ...formData, healthCheckPath: e.target.value })
                }
                placeholder="/health"
              />
            </div>
            <div className="grid gap-2">
              <Label htmlFor="healthInterval">Interval (seconds)</Label>
              <Input
                id="healthInterval"
                type="number"
                value={formData.healthCheckInterval}
                onChange={(e) =>
                  setFormData({
                    ...formData,
                    healthCheckInterval: e.target.value,
                  })
                }
                placeholder="30"
              />
            </div>
          </div>
        )}
      </div>

      <DialogFooter>
        <Button type="button" variant="outline" onClick={onCancel}>
          Cancel
        </Button>
        <Button type="submit" disabled={isSubmitting}>
          {isSubmitting
            ? backend
              ? "Updating..."
              : "Creating..."
            : backend
              ? "Update"
              : "Create"}
        </Button>
      </DialogFooter>
    </form>
  )
}
