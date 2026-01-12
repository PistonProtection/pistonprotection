import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import {
  Filter,
  Plus,
  MoreHorizontal,
  Pencil,
  Trash2,
  GripVertical,
  Shield,
  Globe,
  Clock,
  Code,
  Network,
  Zap,
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
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Switch } from '@/components/ui/switch'
import { Skeleton } from '@/components/ui/skeleton'
import {
  filterRulesQueryOptions,
  backendsQueryOptions,
  apiClient,
  type FilterRule,
} from '@/lib/api'
import { formatNumber } from '@/lib/utils'

const ruleTypeIcons: Record<string, React.ComponentType<{ className?: string }>> = {
  ip: Shield,
  geo: Globe,
  rate: Clock,
  pattern: Code,
  protocol: Network,
  custom: Zap,
}

const ruleTypeLabels: Record<string, string> = {
  ip: 'IP Block',
  geo: 'Geo Block',
  rate: 'Rate Limit',
  pattern: 'Pattern Match',
  protocol: 'Protocol Filter',
  custom: 'Custom Rule',
}

const actionLabels: Record<string, string> = {
  drop: 'Drop',
  ratelimit: 'Rate Limit',
  allow: 'Allow',
  log: 'Log Only',
  challenge: 'Challenge',
}

const actionVariants = {
  drop: 'destructive',
  ratelimit: 'warning',
  allow: 'success',
  log: 'secondary',
  challenge: 'default',
} as const

function FilterFormDialog({
  rule,
  open,
  onOpenChange,
}: {
  rule?: FilterRule
  open: boolean
  onOpenChange: (open: boolean) => void
}) {
  const queryClient = useQueryClient()
  const isEditing = !!rule
  const { data: backends } = useQuery(backendsQueryOptions())

  const [formData, setFormData] = useState<{
    name: string;
    description: string;
    type: 'ip' | 'geo' | 'rate' | 'pattern' | 'protocol' | 'custom';
    action: 'drop' | 'ratelimit' | 'allow' | 'log' | 'challenge';
    priority: number;
    enabled: boolean;
    backendId: string;
    ips: string;
    countries: string;
    rateRequests: number;
    rateWindow: number;
    pattern: string;
    protocol: string;
    expression: string;
  }>({
    name: rule?.name || '',
    description: rule?.description || '',
    type: rule?.type || 'ip',
    action: rule?.action || 'drop',
    priority: rule?.priority || 100,
    enabled: rule?.enabled ?? true,
    backendId: rule?.backendId || '',
    // IP config
    ips: rule?.config?.ips?.join('\n') || '',
    // Geo config
    countries: rule?.config?.countries?.join(', ') || '',
    // Rate limit config
    rateRequests: rule?.config?.rateLimit?.requests || 100,
    rateWindow: rule?.config?.rateLimit?.window || 60,
    // Pattern config
    pattern: rule?.config?.pattern || '',
    // Protocol config
    protocol: rule?.config?.protocol || '',
    // Custom expression
    expression: rule?.config?.expression || '',
  })

  const createMutation = useMutation({
    mutationFn: (data: Parameters<typeof apiClient.createFilterRule>[0]) =>
      apiClient.createFilterRule(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['filterRules'] })
      toast.success('Filter rule created successfully')
      onOpenChange(false)
    },
    onError: (error: Error) => {
      toast.error(`Failed to create filter rule: ${error.message}`)
    },
  })

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: Partial<FilterRule> }) =>
      apiClient.updateFilterRule(id, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['filterRules'] })
      toast.success('Filter rule updated successfully')
      onOpenChange(false)
    },
    onError: (error: Error) => {
      toast.error(`Failed to update filter rule: ${error.message}`)
    },
  })

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()

    const config: FilterRule['config'] = {}

    switch (formData.type) {
      case 'ip':
        config.ips = formData.ips.split('\n').filter(Boolean).map((ip) => ip.trim())
        break
      case 'geo':
        config.countries = formData.countries.split(',').filter(Boolean).map((c) => c.trim())
        break
      case 'rate':
        config.rateLimit = {
          requests: formData.rateRequests,
          window: formData.rateWindow,
        }
        break
      case 'pattern':
        config.pattern = formData.pattern
        break
      case 'protocol':
        config.protocol = formData.protocol
        break
      case 'custom':
        config.expression = formData.expression
        break
    }

    const data = {
      name: formData.name,
      description: formData.description || undefined,
      type: formData.type as FilterRule['type'],
      action: formData.action as FilterRule['action'],
      priority: formData.priority,
      enabled: formData.enabled,
      backendId: formData.backendId || undefined,
      config,
    }

    if (isEditing && rule) {
      updateMutation.mutate({ id: rule.id, data })
    } else {
      createMutation.mutate(data)
    }
  }

  const isPending = createMutation.isPending || updateMutation.isPending

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[550px]">
        <form onSubmit={handleSubmit}>
          <DialogHeader>
            <DialogTitle>
              {isEditing ? 'Edit Filter Rule' : 'Create Filter Rule'}
            </DialogTitle>
            <DialogDescription>
              {isEditing
                ? 'Update the filter rule configuration'
                : 'Configure a new filter rule for traffic filtering'}
            </DialogDescription>
          </DialogHeader>
          <div className="grid max-h-[60vh] gap-4 overflow-y-auto py-4 pr-2">
            <div className="grid gap-2">
              <Label htmlFor="name">Name</Label>
              <Input
                id="name"
                placeholder="Block suspicious IPs"
                value={formData.name}
                onChange={(e) =>
                  setFormData({ ...formData, name: e.target.value })
                }
                required
              />
            </div>
            <div className="grid gap-2">
              <Label htmlFor="description">Description (optional)</Label>
              <Textarea
                id="description"
                placeholder="Describe what this rule does..."
                value={formData.description}
                onChange={(e) =>
                  setFormData({ ...formData, description: e.target.value })
                }
                rows={2}
              />
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="grid gap-2">
                <Label htmlFor="type">Rule Type</Label>
                <Select
                  value={formData.type}
                  onValueChange={(value: 'ip' | 'geo' | 'rate' | 'pattern' | 'protocol' | 'custom') =>
                    setFormData({ ...formData, type: value })
                  }
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="ip">IP Block</SelectItem>
                    <SelectItem value="geo">Geo Block</SelectItem>
                    <SelectItem value="rate">Rate Limit</SelectItem>
                    <SelectItem value="pattern">Pattern Match</SelectItem>
                    <SelectItem value="protocol">Protocol Filter</SelectItem>
                    <SelectItem value="custom">Custom Expression</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="grid gap-2">
                <Label htmlFor="action">Action</Label>
                <Select
                  value={formData.action}
                  onValueChange={(value: 'drop' | 'ratelimit' | 'allow' | 'log' | 'challenge') =>
                    setFormData({ ...formData, action: value })
                  }
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="drop">Drop</SelectItem>
                    <SelectItem value="ratelimit">Rate Limit</SelectItem>
                    <SelectItem value="allow">Allow</SelectItem>
                    <SelectItem value="log">Log Only</SelectItem>
                    <SelectItem value="challenge">Challenge</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>

            {/* Type-specific configuration */}
            {formData.type === 'ip' && (
              <div className="grid gap-2">
                <Label htmlFor="ips">IP Addresses (one per line)</Label>
                <Textarea
                  id="ips"
                  placeholder="192.168.1.100&#10;10.0.0.0/8&#10;2001:db8::/32"
                  value={formData.ips}
                  onChange={(e) =>
                    setFormData({ ...formData, ips: e.target.value })
                  }
                  rows={4}
                />
              </div>
            )}

            {formData.type === 'geo' && (
              <div className="grid gap-2">
                <Label htmlFor="countries">
                  Country Codes (comma-separated)
                </Label>
                <Input
                  id="countries"
                  placeholder="CN, RU, KP"
                  value={formData.countries}
                  onChange={(e) =>
                    setFormData({ ...formData, countries: e.target.value })
                  }
                />
                <p className="text-xs text-muted-foreground">
                  Use ISO 3166-1 alpha-2 country codes
                </p>
              </div>
            )}

            {formData.type === 'rate' && (
              <div className="grid grid-cols-2 gap-4">
                <div className="grid gap-2">
                  <Label htmlFor="rateRequests">Max Requests</Label>
                  <Input
                    id="rateRequests"
                    type="number"
                    min={1}
                    value={formData.rateRequests}
                    onChange={(e) =>
                      setFormData({
                        ...formData,
                        rateRequests: parseInt(e.target.value),
                      })
                    }
                  />
                </div>
                <div className="grid gap-2">
                  <Label htmlFor="rateWindow">Window (seconds)</Label>
                  <Input
                    id="rateWindow"
                    type="number"
                    min={1}
                    value={formData.rateWindow}
                    onChange={(e) =>
                      setFormData({
                        ...formData,
                        rateWindow: parseInt(e.target.value),
                      })
                    }
                  />
                </div>
              </div>
            )}

            {formData.type === 'pattern' && (
              <div className="grid gap-2">
                <Label htmlFor="pattern">Regex Pattern</Label>
                <Input
                  id="pattern"
                  placeholder=".*\\.(php|asp|aspx)$"
                  value={formData.pattern}
                  onChange={(e) =>
                    setFormData({ ...formData, pattern: e.target.value })
                  }
                />
                <p className="text-xs text-muted-foreground">
                  Regular expression to match against request paths
                </p>
              </div>
            )}

            {formData.type === 'protocol' && (
              <div className="grid gap-2">
                <Label htmlFor="protocol">Protocol</Label>
                <Select
                  value={formData.protocol}
                  onValueChange={(value) =>
                    setFormData({ ...formData, protocol: value })
                  }
                >
                  <SelectTrigger>
                    <SelectValue placeholder="Select protocol" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="icmp">ICMP</SelectItem>
                    <SelectItem value="udp">UDP</SelectItem>
                    <SelectItem value="tcp-syn">TCP SYN</SelectItem>
                    <SelectItem value="dns">DNS</SelectItem>
                    <SelectItem value="ntp">NTP</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            )}

            {formData.type === 'custom' && (
              <div className="grid gap-2">
                <Label htmlFor="expression">Custom Expression</Label>
                <Textarea
                  id="expression"
                  placeholder='ip.src in {"192.168.1.0/24"} and http.request.uri.path contains "/admin"'
                  value={formData.expression}
                  onChange={(e) =>
                    setFormData({ ...formData, expression: e.target.value })
                  }
                  rows={4}
                />
                <p className="text-xs text-muted-foreground">
                  Use Wireshark-style filter expressions
                </p>
              </div>
            )}

            <div className="grid grid-cols-2 gap-4">
              <div className="grid gap-2">
                <Label htmlFor="priority">Priority</Label>
                <Input
                  id="priority"
                  type="number"
                  min={1}
                  max={1000}
                  value={formData.priority}
                  onChange={(e) =>
                    setFormData({
                      ...formData,
                      priority: parseInt(e.target.value),
                    })
                  }
                />
                <p className="text-xs text-muted-foreground">
                  Lower = higher priority
                </p>
              </div>
              <div className="grid gap-2">
                <Label htmlFor="backend">Apply to Backend</Label>
                <Select
                  value={formData.backendId}
                  onValueChange={(value) =>
                    setFormData({ ...formData, backendId: value })
                  }
                >
                  <SelectTrigger>
                    <SelectValue placeholder="All backends" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="">All backends</SelectItem>
                    {backends?.map((backend) => (
                      <SelectItem key={backend.id} value={backend.id}>
                        {backend.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </div>

            <div className="flex items-center justify-between rounded-lg border p-3">
              <div className="space-y-0.5">
                <Label htmlFor="enabled">Enabled</Label>
                <p className="text-sm text-muted-foreground">
                  Activate this filter rule
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

function DeleteFilterDialog({
  rule,
  open,
  onOpenChange,
}: {
  rule: FilterRule
  open: boolean
  onOpenChange: (open: boolean) => void
}) {
  const queryClient = useQueryClient()

  const deleteMutation = useMutation({
    mutationFn: () => apiClient.deleteFilterRule(rule.id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['filterRules'] })
      toast.success('Filter rule deleted successfully')
      onOpenChange(false)
    },
    onError: (error: Error) => {
      toast.error(`Failed to delete filter rule: ${error.message}`)
    },
  })

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Delete Filter Rule</DialogTitle>
          <DialogDescription>
            Are you sure you want to delete "{rule.name}"? This action cannot be
            undone.
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

function FilterRuleCard({ rule }: { rule: FilterRule }) {
  const queryClient = useQueryClient()
  const [editOpen, setEditOpen] = useState(false)
  const [deleteOpen, setDeleteOpen] = useState(false)

  const toggleMutation = useMutation({
    mutationFn: () => apiClient.toggleFilterRule(rule.id, !rule.enabled),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['filterRules'] })
      toast.success(rule.enabled ? 'Rule disabled' : 'Rule enabled')
    },
    onError: (error: Error) => {
      toast.error(`Failed to toggle rule: ${error.message}`)
    },
  })

  const TypeIcon = ruleTypeIcons[rule.type] || Filter

  return (
    <>
      <Card className={!rule.enabled ? 'opacity-60' : undefined}>
        <CardContent className="flex items-center gap-4 p-4">
          <div className="cursor-move text-muted-foreground hover:text-foreground">
            <GripVertical className="h-5 w-5" />
          </div>
          <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-muted">
            <TypeIcon className="h-5 w-5" />
          </div>
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2">
              <h3 className="font-medium truncate">{rule.name}</h3>
              <Badge variant="outline" className="text-xs">
                #{rule.priority}
              </Badge>
            </div>
            <p className="text-sm text-muted-foreground truncate">
              {rule.description || ruleTypeLabels[rule.type]}
            </p>
          </div>
          <div className="flex items-center gap-4">
            <div className="text-right">
              <Badge variant={actionVariants[rule.action]}>
                {actionLabels[rule.action]}
              </Badge>
              <p className="mt-1 text-xs text-muted-foreground">
                {formatNumber(rule.matches)} matches
              </p>
            </div>
            <Switch
              checked={rule.enabled}
              onCheckedChange={() => toggleMutation.mutate()}
              disabled={toggleMutation.isPending}
            />
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
          </div>
        </CardContent>
      </Card>

      <FilterFormDialog rule={rule} open={editOpen} onOpenChange={setEditOpen} />
      <DeleteFilterDialog rule={rule} open={deleteOpen} onOpenChange={setDeleteOpen} />
    </>
  )
}

export default function DashboardFilters() {
  const [createOpen, setCreateOpen] = useState(false)
  const { data: rules, isLoading } = useQuery(filterRulesQueryOptions())

  const activeRules = rules?.filter((r) => r.enabled).length || 0
  const totalMatches = rules?.reduce((sum, r) => sum + r.matches, 0) || 0

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Filter Rules</h1>
          <p className="text-muted-foreground">
            Configure traffic filtering and protection rules
          </p>
        </div>
        <Button onClick={() => setCreateOpen(true)}>
          <Plus className="mr-2 h-4 w-4" />
          Add Rule
        </Button>
      </div>

      {/* Summary Cards */}
      <div className="grid gap-4 md:grid-cols-3">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Active Rules</CardTitle>
            <Filter className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {activeRules} / {rules?.length || 0}
            </div>
            <p className="text-xs text-muted-foreground">Currently enforced</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Matches</CardTitle>
            <Shield className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{formatNumber(totalMatches)}</div>
            <p className="text-xs text-muted-foreground">All time rule matches</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Rule Types</CardTitle>
            <Zap className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="flex flex-wrap gap-1">
              {['ip', 'geo', 'rate', 'pattern', 'protocol', 'custom'].map(
                (type) => {
                  const count = rules?.filter((r) => r.type === type).length || 0
                  if (count === 0) return null
                  return (
                    <Badge key={type} variant="secondary" className="text-xs">
                      {ruleTypeLabels[type]}: {count}
                    </Badge>
                  )
                }
              )}
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Filter Rules List */}
      <Card>
        <CardHeader>
          <CardTitle>All Rules</CardTitle>
          <CardDescription>
            Rules are evaluated in priority order (lowest number first). Drag to
            reorder.
          </CardDescription>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="space-y-4">
              {[...Array(3)].map((_, i) => (
                <Skeleton key={i} className="h-20 w-full" />
              ))}
            </div>
          ) : rules?.length ? (
            <div className="space-y-3">
              {[...rules]
                .sort((a, b) => a.priority - b.priority)
                .map((rule) => (
                  <FilterRuleCard key={rule.id} rule={rule} />
                ))}
            </div>
          ) : (
            <div className="flex flex-col items-center justify-center py-12 text-center">
              <Filter className="h-12 w-12 text-muted-foreground" />
              <h3 className="mt-4 text-lg font-semibold">No filter rules</h3>
              <p className="mt-2 text-sm text-muted-foreground">
                Create your first filter rule to start protecting your backends
              </p>
              <Button onClick={() => setCreateOpen(true)} className="mt-4">
                <Plus className="mr-2 h-4 w-4" />
                Add Rule
              </Button>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Create Dialog */}
      <FilterFormDialog open={createOpen} onOpenChange={setCreateOpen} />
    </div>
  )
}
