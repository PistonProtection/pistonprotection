import { useState } from "react"
import { createFileRoute } from "@tanstack/react-router"
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query"
import {
  AlertCircle,
  Ban,
  CheckCircle,
  Globe,
  MoreHorizontal,
  Pencil,
  Plus,
  RefreshCw,
  Shield,
  ShieldAlert,
  ShieldCheck,
  Trash2,
} from "lucide-react"
import { toast } from "sonner"

import { cn } from "@/lib/utils"
import { filtersOptions, type FilterRule } from "@/lib/api"
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
import { Textarea } from "@/components/ui/textarea"
import { Skeleton } from "@/components/ui/skeleton"

export const Route = createFileRoute("/dashboard/filters")({
  component: FiltersPage,
})

const filterTypeIcons: Record<string, React.ComponentType<{ className?: string }>> = {
  rate_limit: ShieldAlert,
  ip_block: Ban,
  geo_block: Globe,
  header_filter: Shield,
  custom: ShieldCheck,
}

const filterTypeLabels: Record<string, string> = {
  rate_limit: "Rate Limit",
  ip_block: "IP Block",
  geo_block: "Geo Block",
  header_filter: "Header Filter",
  custom: "Custom Rule",
}

const actionLabels: Record<string, { label: string; variant: "default" | "destructive" | "secondary" | "outline" }> = {
  allow: { label: "Allow", variant: "default" },
  block: { label: "Block", variant: "destructive" },
  challenge: { label: "Challenge", variant: "secondary" },
  log: { label: "Log Only", variant: "outline" },
}

function FiltersPage() {
  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false)
  const [editingFilter, setEditingFilter] = useState<FilterRule | null>(null)
  const [deleteConfirmFilter, setDeleteConfirmFilter] = useState<FilterRule | null>(
    null
  )

  const queryClient = useQueryClient()
  const { data: filters, isLoading, refetch } = useQuery(filtersOptions)

  const toggleMutation = useMutation({
    mutationFn: ({ id, enabled }: { id: string; enabled: boolean }) => {
      // In dev mode, simulate the toggle
      if (import.meta.env.DEV) {
        return new Promise<void>((resolve) => setTimeout(resolve, 300))
      }
      return Promise.resolve()
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["filters"] })
    },
    onError: () => {
      toast.error("Failed to toggle filter")
    },
  })

  const deleteMutation = useMutation({
    mutationFn: (id: string) => {
      if (import.meta.env.DEV) {
        return new Promise<void>((resolve) => setTimeout(resolve, 500))
      }
      return Promise.resolve()
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["filters"] })
      toast.success("Filter deleted successfully")
      setDeleteConfirmFilter(null)
    },
    onError: () => {
      toast.error("Failed to delete filter")
    },
  })

  const enabledCount = filters?.filter((f) => f.enabled).length || 0
  const totalCount = filters?.length || 0

  return (
    <div className="flex-1 space-y-6 p-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Filter Rules</h1>
          <p className="text-muted-foreground">
            Configure traffic filtering and protection rules.
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
                Add Filter
              </Button>
            </DialogTrigger>
            <DialogContent className="sm:max-w-lg">
              <FilterForm
                onSuccess={() => {
                  setIsCreateDialogOpen(false)
                  queryClient.invalidateQueries({ queryKey: ["filters"] })
                }}
                onCancel={() => setIsCreateDialogOpen(false)}
              />
            </DialogContent>
          </Dialog>
        </div>
      </div>

      {/* Stats Cards */}
      <div className="grid gap-4 md:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardDescription className="text-sm font-medium">
              Total Rules
            </CardDescription>
            <Shield className="size-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{totalCount}</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardDescription className="text-sm font-medium">Active</CardDescription>
            <CheckCircle className="size-4 text-green-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-green-600 dark:text-green-400">
              {enabledCount}
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardDescription className="text-sm font-medium">Disabled</CardDescription>
            <AlertCircle className="size-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-muted-foreground">
              {totalCount - enabledCount}
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardDescription className="text-sm font-medium">
              Blocking Rules
            </CardDescription>
            <Ban className="size-4 text-red-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-red-600 dark:text-red-400">
              {filters?.filter((f) => f.action === "block" && f.enabled).length ||
                0}
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Filters List */}
      <Card>
        <CardHeader>
          <CardTitle>Filter Rules</CardTitle>
          <CardDescription>
            Rules are applied in order of priority (lowest number first).
          </CardDescription>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="space-y-4">
              {[1, 2, 3].map((i) => (
                <div key={i} className="flex items-center gap-4 p-4 border rounded-lg">
                  <Skeleton className="size-10 rounded-lg" />
                  <div className="flex-1 space-y-2">
                    <Skeleton className="h-4 w-40" />
                    <Skeleton className="h-3 w-64" />
                  </div>
                  <Skeleton className="h-6 w-16" />
                  <Skeleton className="h-8 w-12" />
                </div>
              ))}
            </div>
          ) : (
            <div className="space-y-4">
              {filters
                ?.sort((a, b) => a.priority - b.priority)
                .map((filter) => {
                  const Icon = filterTypeIcons[filter.type] || Shield
                  const action = actionLabels[filter.action]
                  return (
                    <div
                      key={filter.id}
                      className={cn(
                        "flex items-center gap-4 p-4 border rounded-lg transition-colors",
                        !filter.enabled && "opacity-60 bg-muted/30"
                      )}
                    >
                      <div
                        className={cn(
                          "flex size-10 items-center justify-center rounded-lg",
                          filter.enabled
                            ? "bg-primary/10 text-primary"
                            : "bg-muted text-muted-foreground"
                        )}
                      >
                        <Icon className="size-5" />
                      </div>

                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2">
                          <p className="font-medium truncate">{filter.name}</p>
                          <Badge variant="outline" className="shrink-0">
                            Priority: {filter.priority}
                          </Badge>
                        </div>
                        <p className="text-sm text-muted-foreground truncate">
                          {filter.description}
                        </p>
                        <div className="flex items-center gap-2 mt-1">
                          <Badge variant="secondary">
                            {filterTypeLabels[filter.type]}
                          </Badge>
                          <Badge variant={action.variant}>{action.label}</Badge>
                        </div>
                      </div>

                      <Switch
                        checked={filter.enabled}
                        onCheckedChange={(enabled) => {
                          toggleMutation.mutate({ id: filter.id, enabled })
                          toast.success(
                            `Filter ${enabled ? "enabled" : "disabled"}`
                          )
                        }}
                      />

                      <DropdownMenu>
                        <DropdownMenuTrigger asChild>
                          <Button variant="ghost" size="icon-sm">
                            <MoreHorizontal className="size-4" />
                            <span className="sr-only">Actions</span>
                          </Button>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent align="end">
                          <DropdownMenuItem
                            onClick={() => setEditingFilter(filter)}
                          >
                            <Pencil className="mr-2 size-4" />
                            Edit
                          </DropdownMenuItem>
                          <DropdownMenuSeparator />
                          <DropdownMenuItem
                            onClick={() => setDeleteConfirmFilter(filter)}
                            className="text-destructive focus:text-destructive"
                          >
                            <Trash2 className="mr-2 size-4" />
                            Delete
                          </DropdownMenuItem>
                        </DropdownMenuContent>
                      </DropdownMenu>
                    </div>
                  )
                })}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Edit Dialog */}
      <Dialog open={!!editingFilter} onOpenChange={() => setEditingFilter(null)}>
        <DialogContent className="sm:max-w-lg">
          {editingFilter && (
            <FilterForm
              filter={editingFilter}
              onSuccess={() => {
                setEditingFilter(null)
                queryClient.invalidateQueries({ queryKey: ["filters"] })
              }}
              onCancel={() => setEditingFilter(null)}
            />
          )}
        </DialogContent>
      </Dialog>

      {/* Delete Confirmation Dialog */}
      <Dialog
        open={!!deleteConfirmFilter}
        onOpenChange={() => setDeleteConfirmFilter(null)}
      >
        <DialogContent className="sm:max-w-md" showCloseButton={false}>
          <DialogHeader>
            <DialogTitle>Delete Filter</DialogTitle>
            <DialogDescription>
              Are you sure you want to delete{" "}
              <strong>{deleteConfirmFilter?.name}</strong>? This action cannot be
              undone.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => setDeleteConfirmFilter(null)}
            >
              Cancel
            </Button>
            <Button
              variant="destructive"
              onClick={() =>
                deleteConfirmFilter && deleteMutation.mutate(deleteConfirmFilter.id)
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

function FilterForm({
  filter,
  onSuccess,
  onCancel,
}: {
  filter?: FilterRule
  onSuccess: () => void
  onCancel: () => void
}) {
  const [formData, setFormData] = useState({
    name: filter?.name || "",
    description: filter?.description || "",
    type: filter?.type || "rate_limit",
    action: filter?.action || "block",
    priority: filter?.priority?.toString() || "10",
    enabled: filter?.enabled ?? true,
  })
  const [isSubmitting, setIsSubmitting] = useState(false)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setIsSubmitting(true)

    try {
      await new Promise((resolve) => setTimeout(resolve, 1000))
      toast.success(
        filter ? "Filter updated successfully" : "Filter created successfully"
      )
      onSuccess()
    } catch {
      toast.error(filter ? "Failed to update filter" : "Failed to create filter")
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <form onSubmit={handleSubmit}>
      <DialogHeader>
        <DialogTitle>{filter ? "Edit Filter" : "Create Filter"}</DialogTitle>
        <DialogDescription>
          {filter
            ? "Update your filter rule configuration."
            : "Create a new traffic filter rule."}
        </DialogDescription>
      </DialogHeader>

      <div className="grid gap-4 py-4">
        <div className="grid gap-2">
          <Label htmlFor="name">Name</Label>
          <Input
            id="name"
            value={formData.name}
            onChange={(e) => setFormData({ ...formData, name: e.target.value })}
            placeholder="Rate Limit - API Endpoints"
            required
          />
        </div>

        <div className="grid gap-2">
          <Label htmlFor="description">Description</Label>
          <Textarea
            id="description"
            value={formData.description}
            onChange={(e) =>
              setFormData({ ...formData, description: e.target.value })
            }
            placeholder="Describe what this filter does..."
            rows={2}
          />
        </div>

        <div className="grid grid-cols-2 gap-4">
          <div className="grid gap-2">
            <Label htmlFor="type">Filter Type</Label>
            <Select
              value={formData.type}
              onValueChange={(value) => setFormData({ ...formData, type: value })}
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="rate_limit">Rate Limit</SelectItem>
                <SelectItem value="ip_block">IP Block</SelectItem>
                <SelectItem value="geo_block">Geo Block</SelectItem>
                <SelectItem value="header_filter">Header Filter</SelectItem>
                <SelectItem value="custom">Custom Rule</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div className="grid gap-2">
            <Label htmlFor="action">Action</Label>
            <Select
              value={formData.action}
              onValueChange={(value) =>
                setFormData({ ...formData, action: value })
              }
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="allow">Allow</SelectItem>
                <SelectItem value="block">Block</SelectItem>
                <SelectItem value="challenge">Challenge</SelectItem>
                <SelectItem value="log">Log Only</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>

        <div className="grid gap-2">
          <Label htmlFor="priority">Priority</Label>
          <Input
            id="priority"
            type="number"
            min="0"
            value={formData.priority}
            onChange={(e) =>
              setFormData({ ...formData, priority: e.target.value })
            }
            placeholder="10"
          />
          <p className="text-xs text-muted-foreground">
            Lower numbers have higher priority and are evaluated first.
          </p>
        </div>

        <div className="flex items-center justify-between">
          <Label htmlFor="enabled">Enable Filter</Label>
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
        <Button type="button" variant="outline" onClick={onCancel}>
          Cancel
        </Button>
        <Button type="submit" disabled={isSubmitting}>
          {isSubmitting
            ? filter
              ? "Updating..."
              : "Creating..."
            : filter
              ? "Update"
              : "Create"}
        </Button>
      </DialogFooter>
    </form>
  )
}
