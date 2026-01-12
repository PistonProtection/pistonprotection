import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import {
  Server,
  Plus,
  MoreHorizontal,
  Pencil,
  Trash2,
  Power,
  PowerOff,
  Activity,
  Shield,
} from 'lucide-react'
import { toast } from 'sonner'

import { Button } from '@/components/ui/button'
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Switch } from '@/components/ui/switch'
import { Skeleton } from '@/components/ui/skeleton'
import { backendsQueryOptions, apiClient, type Backend } from '@/lib/api'
import { formatNumber, formatBytes } from '@/lib/utils'

function BackendFormDialog({
  backend,
  open,
  onOpenChange,
}: {
  backend?: Backend
  open: boolean
  onOpenChange: (open: boolean) => void
}) {
  const queryClient = useQueryClient()
  const isEditing = !!backend

  const [formData, setFormData] = useState<{
    name: string;
    address: string;
    port: number;
    protocol: 'tcp' | 'udp' | 'http' | 'https';
    enabled: boolean;
    healthCheckInterval: number;
    healthCheckTimeout: number;
    healthCheckPath: string;
  }>({
    name: backend?.name || '',
    address: backend?.address || '',
    port: backend?.port || 80,
    protocol: backend?.protocol || 'http',
    enabled: backend?.enabled ?? true,
    healthCheckInterval: backend?.healthCheck?.interval || 30,
    healthCheckTimeout: backend?.healthCheck?.timeout || 5,
    healthCheckPath: backend?.healthCheck?.path || '/health',
  })

  const createMutation = useMutation({
    mutationFn: (data: Parameters<typeof apiClient.createBackend>[0]) =>
      apiClient.createBackend(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['backends'] })
      toast.success('Backend created successfully')
      onOpenChange(false)
    },
    onError: (error: Error) => {
      toast.error(`Failed to create backend: ${error.message}`)
    },
  })

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: Partial<Backend> }) =>
      apiClient.updateBackend(id, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['backends'] })
      toast.success('Backend updated successfully')
      onOpenChange(false)
    },
    onError: (error: Error) => {
      toast.error(`Failed to update backend: ${error.message}`)
    },
  })

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    const data = {
      name: formData.name,
      address: formData.address,
      port: formData.port,
      protocol: formData.protocol as 'tcp' | 'udp' | 'http' | 'https',
      enabled: formData.enabled,
      healthCheck: {
        interval: formData.healthCheckInterval,
        timeout: formData.healthCheckTimeout,
        path: formData.healthCheckPath,
      },
    }

    if (isEditing && backend) {
      updateMutation.mutate({ id: backend.id, data })
    } else {
      createMutation.mutate(data)
    }
  }

  const isPending = createMutation.isPending || updateMutation.isPending

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[500px]">
        <form onSubmit={handleSubmit}>
          <DialogHeader>
            <DialogTitle>
              {isEditing ? 'Edit Backend' : 'Add New Backend'}
            </DialogTitle>
            <DialogDescription>
              {isEditing
                ? 'Update the backend server configuration'
                : 'Configure a new backend server to protect'}
            </DialogDescription>
          </DialogHeader>
          <div className="grid gap-4 py-4">
            <div className="grid gap-2">
              <Label htmlFor="name">Name</Label>
              <Input
                id="name"
                placeholder="My Backend Server"
                value={formData.name}
                onChange={(e) =>
                  setFormData({ ...formData, name: e.target.value })
                }
                required
              />
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="grid gap-2">
                <Label htmlFor="address">Address</Label>
                <Input
                  id="address"
                  placeholder="192.168.1.100"
                  value={formData.address}
                  onChange={(e) =>
                    setFormData({ ...formData, address: e.target.value })
                  }
                  required
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="port">Port</Label>
                <Input
                  id="port"
                  type="number"
                  min={1}
                  max={65535}
                  value={formData.port}
                  onChange={(e) =>
                    setFormData({ ...formData, port: parseInt(e.target.value) })
                  }
                  required
                />
              </div>
            </div>
            <div className="grid gap-2">
              <Label htmlFor="protocol">Protocol</Label>
              <Select
                value={formData.protocol}
                onValueChange={(value: 'tcp' | 'udp' | 'http' | 'https') =>
                  setFormData({ ...formData, protocol: value })
                }
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="http">HTTP</SelectItem>
                  <SelectItem value="https">HTTPS</SelectItem>
                  <SelectItem value="tcp">TCP</SelectItem>
                  <SelectItem value="udp">UDP</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="grid grid-cols-3 gap-4">
              <div className="grid gap-2">
                <Label htmlFor="healthInterval">Health Interval (s)</Label>
                <Input
                  id="healthInterval"
                  type="number"
                  min={5}
                  max={300}
                  value={formData.healthCheckInterval}
                  onChange={(e) =>
                    setFormData({
                      ...formData,
                      healthCheckInterval: parseInt(e.target.value),
                    })
                  }
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="healthTimeout">Timeout (s)</Label>
                <Input
                  id="healthTimeout"
                  type="number"
                  min={1}
                  max={30}
                  value={formData.healthCheckTimeout}
                  onChange={(e) =>
                    setFormData({
                      ...formData,
                      healthCheckTimeout: parseInt(e.target.value),
                    })
                  }
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="healthPath">Health Path</Label>
                <Input
                  id="healthPath"
                  placeholder="/health"
                  value={formData.healthCheckPath}
                  onChange={(e) =>
                    setFormData({ ...formData, healthCheckPath: e.target.value })
                  }
                />
              </div>
            </div>
            <div className="flex items-center justify-between rounded-lg border p-3">
              <div className="space-y-0.5">
                <Label htmlFor="enabled">Enabled</Label>
                <p className="text-sm text-muted-foreground">
                  Enable protection for this backend
                </p>
              </div>
              <Switch
                id="enabled"
                checked={formData.enabled}
                onCheckedChange={(checked) =>
                  setFormData({ ...formData, enabled: checked })
                }
              />
            </div>
          </div>
          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={() => onOpenChange(false)}
            >
              Cancel
            </Button>
            <Button type="submit" disabled={isPending}>
              {isPending ? 'Saving...' : isEditing ? 'Save Changes' : 'Create'}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

function DeleteBackendDialog({
  backend,
  open,
  onOpenChange,
}: {
  backend: Backend
  open: boolean
  onOpenChange: (open: boolean) => void
}) {
  const queryClient = useQueryClient()

  const deleteMutation = useMutation({
    mutationFn: () => apiClient.deleteBackend(backend.id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['backends'] })
      toast.success('Backend deleted successfully')
      onOpenChange(false)
    },
    onError: (error: Error) => {
      toast.error(`Failed to delete backend: ${error.message}`)
    },
  })

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Delete Backend</DialogTitle>
          <DialogDescription>
            Are you sure you want to delete "{backend.name}"? This action cannot
            be undone and will remove all associated filter rules.
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button
            variant="destructive"
            onClick={() => deleteMutation.mutate()}
            disabled={deleteMutation.isPending}
          >
            {deleteMutation.isPending ? 'Deleting...' : 'Delete'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

function BackendRow({ backend }: { backend: Backend }) {
  const queryClient = useQueryClient()
  const [editOpen, setEditOpen] = useState(false)
  const [deleteOpen, setDeleteOpen] = useState(false)

  const toggleMutation = useMutation({
    mutationFn: () => apiClient.toggleBackend(backend.id, !backend.enabled),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['backends'] })
      toast.success(
        backend.enabled ? 'Backend disabled' : 'Backend enabled'
      )
    },
    onError: (error: Error) => {
      toast.error(`Failed to toggle backend: ${error.message}`)
    },
  })

  const statusVariant: 'success' | 'warning' | 'destructive' =
    backend.status === 'healthy'
      ? 'success'
      : backend.status === 'degraded'
      ? 'warning'
      : 'destructive'

  return (
    <>
      <TableRow>
        <TableCell>
          <div className="flex items-center gap-3">
            <div
              className={`h-2 w-2 rounded-full ${
                backend.enabled
                  ? backend.status === 'healthy'
                    ? 'bg-success'
                    : backend.status === 'degraded'
                    ? 'bg-warning'
                    : 'bg-destructive'
                  : 'bg-muted'
              }`}
            />
            <div>
              <p className="font-medium">{backend.name}</p>
              <p className="text-sm text-muted-foreground">
                {backend.address}:{backend.port}
              </p>
            </div>
          </div>
        </TableCell>
        <TableCell>
          <Badge variant="outline">{backend.protocol.toUpperCase()}</Badge>
        </TableCell>
        <TableCell>
          <Badge variant={backend.enabled ? statusVariant : 'secondary'}>
            {backend.enabled ? backend.status : 'disabled'}
          </Badge>
        </TableCell>
        <TableCell className="text-right">
          {formatNumber(backend.stats.requests)}
        </TableCell>
        <TableCell className="text-right">
          <span className="text-destructive">
            {formatNumber(backend.stats.blocked)}
          </span>
        </TableCell>
        <TableCell className="text-right">
          {formatBytes(backend.stats.bytesIn + backend.stats.bytesOut)}
        </TableCell>
        <TableCell>
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="ghost" size="icon">
                <MoreHorizontal className="h-4 w-4" />
                <span className="sr-only">Open menu</span>
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuItem onClick={() => setEditOpen(true)}>
                <Pencil className="mr-2 h-4 w-4" />
                Edit
              </DropdownMenuItem>
              <DropdownMenuItem
                onClick={() => toggleMutation.mutate()}
                disabled={toggleMutation.isPending}
              >
                {backend.enabled ? (
                  <>
                    <PowerOff className="mr-2 h-4 w-4" />
                    Disable
                  </>
                ) : (
                  <>
                    <Power className="mr-2 h-4 w-4" />
                    Enable
                  </>
                )}
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem
                onClick={() => setDeleteOpen(true)}
                className="text-destructive focus:text-destructive"
              >
                <Trash2 className="mr-2 h-4 w-4" />
                Delete
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </TableCell>
      </TableRow>

      <BackendFormDialog
        backend={backend}
        open={editOpen}
        onOpenChange={setEditOpen}
      />
      <DeleteBackendDialog
        backend={backend}
        open={deleteOpen}
        onOpenChange={setDeleteOpen}
      />
    </>
  )
}

export default function DashboardBackends() {
  const [createOpen, setCreateOpen] = useState(false)
  const { data: backends, isLoading } = useQuery(backendsQueryOptions())

  const totalRequests = backends?.reduce((sum, b) => sum + b.stats.requests, 0) || 0
  const totalBlocked = backends?.reduce((sum, b) => sum + b.stats.blocked, 0) || 0
  const activeBackends = backends?.filter((b) => b.enabled).length || 0

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Backends</h1>
          <p className="text-muted-foreground">
            Manage your protected backend servers
          </p>
        </div>
        <Button onClick={() => setCreateOpen(true)}>
          <Plus className="mr-2 h-4 w-4" />
          Add Backend
        </Button>
      </div>

      {/* Summary Cards */}
      <div className="grid gap-4 md:grid-cols-3">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Active Backends</CardTitle>
            <Server className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {activeBackends} / {backends?.length || 0}
            </div>
            <p className="text-xs text-muted-foreground">
              Currently protected servers
            </p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Requests</CardTitle>
            <Activity className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{formatNumber(totalRequests)}</div>
            <p className="text-xs text-muted-foreground">Across all backends</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Threats Blocked</CardTitle>
            <Shield className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-destructive">
              {formatNumber(totalBlocked)}
            </div>
            <p className="text-xs text-muted-foreground">
              {totalRequests > 0
                ? `${((totalBlocked / totalRequests) * 100).toFixed(1)}% block rate`
                : 'No traffic yet'}
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Backends Table */}
      <Card>
        <CardHeader>
          <CardTitle>All Backends</CardTitle>
          <CardDescription>
            A list of all configured backend servers and their status
          </CardDescription>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="space-y-4">
              {[...Array(3)].map((_, i) => (
                <Skeleton key={i} className="h-16 w-full" />
              ))}
            </div>
          ) : backends?.length ? (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Backend</TableHead>
                  <TableHead>Protocol</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead className="text-right">Requests</TableHead>
                  <TableHead className="text-right">Blocked</TableHead>
                  <TableHead className="text-right">Bandwidth</TableHead>
                  <TableHead className="w-[50px]"></TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {backends.map((backend) => (
                  <BackendRow key={backend.id} backend={backend} />
                ))}
              </TableBody>
            </Table>
          ) : (
            <div className="flex flex-col items-center justify-center py-12 text-center">
              <Server className="h-12 w-12 text-muted-foreground" />
              <h3 className="mt-4 text-lg font-semibold">No backends configured</h3>
              <p className="mt-2 text-sm text-muted-foreground">
                Add your first backend server to start protecting it
              </p>
              <Button onClick={() => setCreateOpen(true)} className="mt-4">
                <Plus className="mr-2 h-4 w-4" />
                Add Backend
              </Button>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Create Dialog */}
      <BackendFormDialog open={createOpen} onOpenChange={setCreateOpen} />
    </div>
  )
}
