import { useQuery } from '@tanstack/react-query'
import {
  Shield,
  ShieldOff,
  Activity,
  Server,
  ArrowUpRight,
  ArrowDownRight,
  Zap,
  Globe,
  Clock,
} from 'lucide-react'

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Progress } from '@/components/ui/progress'
import { Skeleton } from '@/components/ui/skeleton'
import { metricsQueryOptions, backendsQueryOptions, recentEventsQueryOptions } from '@/lib/api'
import { formatNumber, formatBytes, formatDuration } from '@/lib/utils'

function MetricCard({
  title,
  value,
  description,
  icon: Icon,
  trend,
  trendValue,
  isLoading,
}: {
  title: string
  value: string | number
  description?: string
  icon: React.ComponentType<{ className?: string }>
  trend?: 'up' | 'down'
  trendValue?: string
  isLoading?: boolean
}) {
  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle className="text-sm font-medium">{title}</CardTitle>
        <Icon className="h-4 w-4 text-muted-foreground" />
      </CardHeader>
      <CardContent>
        {isLoading ? (
          <Skeleton className="h-8 w-24" />
        ) : (
          <>
            <div className="text-2xl font-bold">{value}</div>
            <div className="flex items-center gap-1 text-xs text-muted-foreground">
              {trend && (
                <span
                  className={
                    trend === 'up' ? 'text-success flex items-center' : 'text-destructive flex items-center'
                  }
                >
                  {trend === 'up' ? (
                    <ArrowUpRight className="h-3 w-3" />
                  ) : (
                    <ArrowDownRight className="h-3 w-3" />
                  )}
                  {trendValue}
                </span>
              )}
              {description}
            </div>
          </>
        )}
      </CardContent>
    </Card>
  )
}

function BackendStatusCard({
  isLoading,
  backends,
}: {
  isLoading: boolean
  backends?: Array<{
    id: string
    name: string
    status: 'healthy' | 'degraded' | 'offline'
    enabled: boolean
    stats: { requests: number; blocked: number }
  }>
}) {
  const statusCounts = {
    healthy: backends?.filter((b) => b.status === 'healthy' && b.enabled).length || 0,
    degraded: backends?.filter((b) => b.status === 'degraded' && b.enabled).length || 0,
    offline: backends?.filter((b) => b.status === 'offline' || !b.enabled).length || 0,
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Server className="h-5 w-5" />
          Backend Status
        </CardTitle>
        <CardDescription>Current status of protected backends</CardDescription>
      </CardHeader>
      <CardContent>
        {isLoading ? (
          <div className="space-y-3">
            <Skeleton className="h-4 w-full" />
            <Skeleton className="h-4 w-full" />
            <Skeleton className="h-4 w-full" />
          </div>
        ) : (
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <div className="h-3 w-3 rounded-full bg-success" />
                <span className="text-sm">Healthy</span>
              </div>
              <span className="font-medium">{statusCounts.healthy}</span>
            </div>
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <div className="h-3 w-3 rounded-full bg-warning" />
                <span className="text-sm">Degraded</span>
              </div>
              <span className="font-medium">{statusCounts.degraded}</span>
            </div>
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <div className="h-3 w-3 rounded-full bg-destructive" />
                <span className="text-sm">Offline</span>
              </div>
              <span className="font-medium">{statusCounts.offline}</span>
            </div>
            <Progress
              value={
                backends?.length
                  ? (statusCounts.healthy / backends.length) * 100
                  : 0
              }
              className="h-2"
            />
            <p className="text-xs text-muted-foreground">
              {backends?.length || 0} total backends configured
            </p>
          </div>
        )}
      </CardContent>
    </Card>
  )
}

function RecentEventsCard({
  isLoading,
  events,
}: {
  isLoading: boolean
  events?: Array<{
    id: string
    timestamp: string
    sourceIp: string
    action: string
    ruleName?: string
    country?: string
  }>
}) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Activity className="h-5 w-5" />
          Recent Events
        </CardTitle>
        <CardDescription>Latest traffic events and actions</CardDescription>
      </CardHeader>
      <CardContent>
        {isLoading ? (
          <div className="space-y-3">
            {[...Array(5)].map((_, i) => (
              <Skeleton key={i} className="h-12 w-full" />
            ))}
          </div>
        ) : events?.length ? (
          <div className="space-y-3">
            {events.slice(0, 5).map((event) => (
              <div
                key={event.id}
                className="flex items-center justify-between rounded-lg border p-3"
              >
                <div className="flex items-center gap-3">
                  <div
                    className={`h-2 w-2 rounded-full ${
                      event.action === 'drop'
                        ? 'bg-destructive'
                        : event.action === 'ratelimit'
                        ? 'bg-warning'
                        : 'bg-success'
                    }`}
                  />
                  <div>
                    <p className="text-sm font-medium">{event.sourceIp}</p>
                    <p className="text-xs text-muted-foreground">
                      {event.ruleName || 'No rule matched'}
                    </p>
                  </div>
                </div>
                <div className="text-right">
                  <Badge
                    variant={
                      event.action === 'drop'
                        ? 'destructive'
                        : event.action === 'ratelimit'
                        ? 'warning'
                        : 'success'
                    }
                  >
                    {event.action}
                  </Badge>
                  {event.country && (
                    <p className="mt-1 text-xs text-muted-foreground">
                      {event.country}
                    </p>
                  )}
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="flex flex-col items-center justify-center py-8 text-center">
            <Activity className="h-10 w-10 text-muted-foreground" />
            <p className="mt-2 text-sm text-muted-foreground">No recent events</p>
          </div>
        )}
      </CardContent>
    </Card>
  )
}

export default function DashboardOverview() {
  const { data: metrics, isLoading: metricsLoading } = useQuery(metricsQueryOptions())
  const { data: backends, isLoading: backendsLoading } = useQuery(backendsQueryOptions())
  const { data: events, isLoading: eventsLoading } = useQuery(recentEventsQueryOptions(10))

  const blockRate = metrics
    ? ((metrics.blockedRequests / (metrics.totalRequests || 1)) * 100).toFixed(1)
    : '0'

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div>
        <h1 className="text-3xl font-bold tracking-tight">Dashboard</h1>
        <p className="text-muted-foreground">
          Monitor your DDoS protection status and traffic metrics
        </p>
      </div>

      {/* Metrics Grid */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <MetricCard
          title="Total Requests"
          value={formatNumber(metrics?.totalRequests || 0)}
          description="Last 24 hours"
          icon={Globe}
          trend="up"
          trendValue="12%"
          isLoading={metricsLoading}
        />
        <MetricCard
          title="Blocked Threats"
          value={formatNumber(metrics?.blockedRequests || 0)}
          description={`${blockRate}% block rate`}
          icon={ShieldOff}
          trend="down"
          trendValue="3%"
          isLoading={metricsLoading}
        />
        <MetricCard
          title="Active Connections"
          value={formatNumber(metrics?.activeConnections || 0)}
          description={`Peak: ${formatNumber(metrics?.peakConnections || 0)}`}
          icon={Zap}
          isLoading={metricsLoading}
        />
        <MetricCard
          title="Avg Latency"
          value={formatDuration(metrics?.avgLatency || 0)}
          description={`P99: ${formatDuration(metrics?.p99Latency || 0)}`}
          icon={Clock}
          isLoading={metricsLoading}
        />
      </div>

      {/* Secondary Metrics */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <MetricCard
          title="Bandwidth In"
          value={formatBytes(metrics?.bytesIn || 0)}
          description="Total inbound"
          icon={ArrowDownRight}
          isLoading={metricsLoading}
        />
        <MetricCard
          title="Bandwidth Out"
          value={formatBytes(metrics?.bytesOut || 0)}
          description="Total outbound"
          icon={ArrowUpRight}
          isLoading={metricsLoading}
        />
        <MetricCard
          title="Passed Requests"
          value={formatNumber(metrics?.passedRequests || 0)}
          description="Legitimate traffic"
          icon={Shield}
          trend="up"
          trendValue="8%"
          isLoading={metricsLoading}
        />
        <MetricCard
          title="Challenged"
          value={formatNumber(metrics?.challengedRequests || 0)}
          description="Captcha challenges"
          icon={Activity}
          isLoading={metricsLoading}
        />
      </div>

      {/* Status Cards */}
      <div className="grid gap-4 md:grid-cols-2">
        <BackendStatusCard isLoading={backendsLoading} backends={backends} />
        <RecentEventsCard isLoading={eventsLoading} events={events} />
      </div>
    </div>
  )
}
