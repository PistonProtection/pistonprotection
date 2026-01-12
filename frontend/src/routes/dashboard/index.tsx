import { createFileRoute } from "@tanstack/react-router"
import { useQuery } from "@tanstack/react-query"
import {
  Activity,
  ArrowDown,
  ArrowUp,
  Globe,
  Server,
  Shield,
  TrendingDown,
  TrendingUp,
  Zap,
} from "lucide-react"
import {
  Area,
  AreaChart,
  ResponsiveContainer,
  XAxis,
  YAxis,
  Tooltip as RechartsTooltip,
} from "recharts"

import { cn } from "@/lib/utils"
import {
  dashboardMetricsOptions,
  backendsOptions,
  analyticsOptions,
} from "@/lib/api"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Progress, ProgressLabel, ProgressValue } from "@/components/ui/progress"
import { Skeleton } from "@/components/ui/skeleton"

export const Route = createFileRoute("/dashboard/")({
  component: DashboardOverview,
})

function formatNumber(num: number): string {
  if (num >= 1_000_000) {
    return (num / 1_000_000).toFixed(1) + "M"
  }
  if (num >= 1_000) {
    return (num / 1_000).toFixed(1) + "K"
  }
  return num.toString()
}

function formatBytes(bytes: number): string {
  if (bytes >= 1024 * 1024 * 1024) {
    return (bytes / (1024 * 1024 * 1024)).toFixed(2) + " GB"
  }
  if (bytes >= 1024 * 1024) {
    return (bytes / (1024 * 1024)).toFixed(2) + " MB"
  }
  if (bytes >= 1024) {
    return (bytes / 1024).toFixed(2) + " KB"
  }
  return bytes + " B"
}

function MetricCard({
  title,
  value,
  change,
  icon: Icon,
  trend,
  loading,
}: {
  title: string
  value: string
  change?: number
  icon: React.ComponentType<{ className?: string }>
  trend?: "up" | "down"
  loading?: boolean
}) {
  if (loading) {
    return (
      <Card>
        <CardHeader className="flex flex-row items-center justify-between pb-2">
          <Skeleton className="h-4 w-24" />
          <Skeleton className="size-4" />
        </CardHeader>
        <CardContent>
          <Skeleton className="h-8 w-32 mb-2" />
          <Skeleton className="h-4 w-20" />
        </CardContent>
      </Card>
    )
  }

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between pb-2">
        <CardDescription className="text-sm font-medium">{title}</CardDescription>
        <Icon className="size-4 text-muted-foreground" />
      </CardHeader>
      <CardContent>
        <div className="text-2xl font-bold">{value}</div>
        {change !== undefined && (
          <div
            className={cn(
              "flex items-center text-xs",
              trend === "up" ? "text-green-600 dark:text-green-400" : "",
              trend === "down" ? "text-red-600 dark:text-red-400" : ""
            )}
          >
            {trend === "up" ? (
              <TrendingUp className="mr-1 size-3" />
            ) : trend === "down" ? (
              <TrendingDown className="mr-1 size-3" />
            ) : null}
            {change > 0 ? "+" : ""}
            {change.toFixed(1)}% from last period
          </div>
        )}
      </CardContent>
    </Card>
  )
}

function DashboardOverview() {
  const { data: metrics, isLoading: metricsLoading } = useQuery(
    dashboardMetricsOptions
  )
  const { data: backends, isLoading: backendsLoading } = useQuery(backendsOptions)

  // Get analytics for the last 24 hours
  const now = new Date()
  const dayAgo = new Date(now.getTime() - 24 * 60 * 60 * 1000)
  const { data: analyticsData, isLoading: analyticsLoading } = useQuery(
    analyticsOptions({
      startDate: dayAgo.toISOString(),
      endDate: now.toISOString(),
      granularity: "hour",
    })
  )

  const chartData =
    analyticsData?.map((item) => ({
      time: new Date(item.timestamp).toLocaleTimeString([], {
        hour: "2-digit",
        minute: "2-digit",
      }),
      requests: item.requests,
      blocked: item.blocked,
    })) || []

  return (
    <div className="flex-1 space-y-6 p-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Dashboard</h1>
          <p className="text-muted-foreground">
            Monitor your DDoS protection status and traffic metrics.
          </p>
        </div>
      </div>

      {/* Metrics Grid */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <MetricCard
          title="Total Requests"
          value={metrics ? formatNumber(metrics.totalRequests) : "0"}
          change={metrics?.requestsChange}
          trend={metrics?.requestsChange && metrics.requestsChange > 0 ? "up" : "down"}
          icon={Activity}
          loading={metricsLoading}
        />
        <MetricCard
          title="Attacks Blocked"
          value={metrics ? formatNumber(metrics.blockedRequests) : "0"}
          change={metrics?.blockedChange}
          trend={metrics?.blockedChange && metrics.blockedChange < 0 ? "up" : "down"}
          icon={Shield}
          loading={metricsLoading}
        />
        <MetricCard
          title="Bandwidth Used"
          value={metrics ? formatBytes(metrics.bandwidth) : "0 B"}
          change={metrics?.bandwidthChange}
          trend={metrics?.bandwidthChange && metrics.bandwidthChange > 0 ? "up" : "down"}
          icon={Globe}
          loading={metricsLoading}
        />
        <MetricCard
          title="Avg Latency"
          value={metrics ? `${metrics.avgLatency}ms` : "0ms"}
          change={metrics?.latencyChange}
          trend={metrics?.latencyChange && metrics.latencyChange < 0 ? "up" : "down"}
          icon={Zap}
          loading={metricsLoading}
        />
      </div>

      {/* Charts and Status */}
      <div className="grid gap-4 lg:grid-cols-7">
        {/* Traffic Chart */}
        <Card className="lg:col-span-4">
          <CardHeader>
            <CardTitle>Traffic Overview</CardTitle>
            <CardDescription>
              Request volume over the last 24 hours
            </CardDescription>
          </CardHeader>
          <CardContent>
            {analyticsLoading ? (
              <Skeleton className="h-[300px] w-full" />
            ) : (
              <div className="h-[300px]">
                <ResponsiveContainer width="100%" height="100%">
                  <AreaChart data={chartData}>
                    <defs>
                      <linearGradient id="requests" x1="0" y1="0" x2="0" y2="1">
                        <stop
                          offset="5%"
                          stopColor="hsl(var(--chart-1))"
                          stopOpacity={0.3}
                        />
                        <stop
                          offset="95%"
                          stopColor="hsl(var(--chart-1))"
                          stopOpacity={0}
                        />
                      </linearGradient>
                      <linearGradient id="blocked" x1="0" y1="0" x2="0" y2="1">
                        <stop
                          offset="5%"
                          stopColor="hsl(var(--destructive))"
                          stopOpacity={0.3}
                        />
                        <stop
                          offset="95%"
                          stopColor="hsl(var(--destructive))"
                          stopOpacity={0}
                        />
                      </linearGradient>
                    </defs>
                    <XAxis
                      dataKey="time"
                      stroke="hsl(var(--muted-foreground))"
                      fontSize={12}
                      tickLine={false}
                      axisLine={false}
                    />
                    <YAxis
                      stroke="hsl(var(--muted-foreground))"
                      fontSize={12}
                      tickLine={false}
                      axisLine={false}
                      tickFormatter={(value) => formatNumber(value)}
                    />
                    <RechartsTooltip
                      contentStyle={{
                        backgroundColor: "hsl(var(--popover))",
                        border: "1px solid hsl(var(--border))",
                        borderRadius: "8px",
                        color: "hsl(var(--popover-foreground))",
                      }}
                      formatter={(value: number) => [formatNumber(value), ""]}
                    />
                    <Area
                      type="monotone"
                      dataKey="requests"
                      stroke="hsl(var(--chart-1))"
                      fill="url(#requests)"
                      strokeWidth={2}
                      name="Requests"
                    />
                    <Area
                      type="monotone"
                      dataKey="blocked"
                      stroke="hsl(var(--destructive))"
                      fill="url(#blocked)"
                      strokeWidth={2}
                      name="Blocked"
                    />
                  </AreaChart>
                </ResponsiveContainer>
              </div>
            )}
          </CardContent>
        </Card>

        {/* Backend Status */}
        <Card className="lg:col-span-3">
          <CardHeader>
            <CardTitle>Backend Status</CardTitle>
            <CardDescription>
              Health status of your backend servers
            </CardDescription>
          </CardHeader>
          <CardContent>
            {backendsLoading ? (
              <div className="space-y-4">
                {[1, 2, 3].map((i) => (
                  <div key={i} className="flex items-center gap-4">
                    <Skeleton className="size-2 rounded-full" />
                    <Skeleton className="h-4 flex-1" />
                    <Skeleton className="h-5 w-16" />
                  </div>
                ))}
              </div>
            ) : (
              <div className="space-y-4">
                {backends?.map((backend) => (
                  <div
                    key={backend.id}
                    className="flex items-center justify-between"
                  >
                    <div className="flex items-center gap-3">
                      <div
                        className={cn(
                          "size-2 rounded-full",
                          backend.status === "healthy"
                            ? "bg-green-500"
                            : backend.status === "unhealthy"
                              ? "bg-red-500"
                              : "bg-yellow-500"
                        )}
                      />
                      <div>
                        <p className="font-medium text-sm">{backend.name}</p>
                        <p className="text-xs text-muted-foreground">
                          {backend.host}:{backend.port}
                        </p>
                      </div>
                    </div>
                    <Badge
                      variant={
                        backend.status === "healthy"
                          ? "default"
                          : backend.status === "unhealthy"
                            ? "destructive"
                            : "secondary"
                      }
                    >
                      {backend.status}
                    </Badge>
                  </div>
                ))}
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      {/* Quick Stats */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        <Card>
          <CardHeader>
            <CardTitle className="text-base">Protection Status</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center justify-between">
              <span className="text-sm text-muted-foreground">Active Filters</span>
              <span className="font-medium">{metrics?.activeFilters || 0}</span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm text-muted-foreground">
                Active Backends
              </span>
              <span className="font-medium">
                {metrics?.activeBackends || 0} / {metrics?.totalBackends || 0}
              </span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm text-muted-foreground">
                Attacks Blocked Today
              </span>
              <span className="font-medium text-green-600 dark:text-green-400">
                {metrics?.attacksBlocked || 0}
              </span>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="text-base">Request Distribution</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <div className="flex items-center justify-between text-sm">
                <span className="text-muted-foreground">Allowed</span>
                <span className="font-medium">
                  {metrics
                    ? (
                        (metrics.allowedRequests / metrics.totalRequests) *
                        100
                      ).toFixed(1)
                    : 0}
                  %
                </span>
              </div>
              <Progress
                value={
                  metrics
                    ? (metrics.allowedRequests / metrics.totalRequests) * 100
                    : 0
                }
              />
            </div>
            <div className="space-y-2">
              <div className="flex items-center justify-between text-sm">
                <span className="text-muted-foreground">Blocked</span>
                <span className="font-medium text-destructive">
                  {metrics
                    ? (
                        (metrics.blockedRequests / metrics.totalRequests) *
                        100
                      ).toFixed(1)
                    : 0}
                  %
                </span>
              </div>
              <Progress
                value={
                  metrics
                    ? (metrics.blockedRequests / metrics.totalRequests) * 100
                    : 0
                }
                className="[&>[data-slot=progress-track]>[data-slot=progress-indicator]]:bg-destructive"
              />
            </div>
            <div className="space-y-2">
              <div className="flex items-center justify-between text-sm">
                <span className="text-muted-foreground">Challenged</span>
                <span className="font-medium text-yellow-600 dark:text-yellow-400">
                  {metrics
                    ? (
                        (metrics.challengedRequests / metrics.totalRequests) *
                        100
                      ).toFixed(1)
                    : 0}
                  %
                </span>
              </div>
              <Progress
                value={
                  metrics
                    ? (metrics.challengedRequests / metrics.totalRequests) * 100
                    : 0
                }
                className="[&>[data-slot=progress-track]>[data-slot=progress-indicator]]:bg-yellow-500"
              />
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="text-base">System Health</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center gap-3">
              <div className="flex size-10 items-center justify-center rounded-full bg-green-100 dark:bg-green-900/30">
                <Shield className="size-5 text-green-600 dark:text-green-400" />
              </div>
              <div>
                <p className="font-medium">All Systems Operational</p>
                <p className="text-xs text-muted-foreground">
                  Last checked: Just now
                </p>
              </div>
            </div>
            <div className="space-y-2 text-sm">
              <div className="flex items-center justify-between">
                <span className="text-muted-foreground">API Gateway</span>
                <Badge variant="default" className="bg-green-600">
                  Healthy
                </Badge>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-muted-foreground">Filter Engine</span>
                <Badge variant="default" className="bg-green-600">
                  Healthy
                </Badge>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-muted-foreground">Analytics</span>
                <Badge variant="default" className="bg-green-600">
                  Healthy
                </Badge>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
