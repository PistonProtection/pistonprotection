import { useState, useMemo } from 'react'
import {
  BarChart3,
  Globe,
  Shield,
  TrendingUp,
  TrendingDown,
  Calendar,
} from 'lucide-react'
import {
  LineChart,
  Line,
  AreaChart,
  Area,
  BarChart,
  Bar,
  PieChart,
  Pie,
  Cell,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from 'recharts'

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { formatNumber, formatBytes } from '@/lib/utils'

const PIE_COLORS = ['#22c55e', '#ef4444', '#f59e0b', '#3b82f6', '#8b5cf6', '#ec4899']

function TimeRangeSelector({
  value,
  onChange,
}: {
  value: string
  onChange: (value: string) => void
}) {
  return (
    <Select value={value} onValueChange={onChange}>
      <SelectTrigger className="w-[180px]">
        <Calendar className="mr-2 h-4 w-4" />
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value="1h">Last Hour</SelectItem>
        <SelectItem value="24h">Last 24 Hours</SelectItem>
        <SelectItem value="7d">Last 7 Days</SelectItem>
        <SelectItem value="30d">Last 30 Days</SelectItem>
        <SelectItem value="90d">Last 90 Days</SelectItem>
      </SelectContent>
    </Select>
  )
}

function TrafficChart({ timeRange }: { timeRange: string }) {
  // Generate mock data based on time range
  const data = useMemo(() => {
    const now = Date.now()
    const points = timeRange === '1h' ? 12 : timeRange === '24h' ? 24 : timeRange === '7d' ? 7 : 30
    const interval = timeRange === '1h' ? 5 * 60 * 1000 : timeRange === '24h' ? 60 * 60 * 1000 : 24 * 60 * 60 * 1000

    return Array.from({ length: points }, (_, i) => {
      const timestamp = new Date(now - (points - i - 1) * interval)
      const baseRequests = 10000 + Math.random() * 5000
      const blocked = baseRequests * (0.05 + Math.random() * 0.1)
      return {
        time: timeRange === '1h'
          ? timestamp.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
          : timeRange === '24h'
          ? timestamp.toLocaleTimeString([], { hour: '2-digit' })
          : timestamp.toLocaleDateString([], { month: 'short', day: 'numeric' }),
        requests: Math.round(baseRequests),
        blocked: Math.round(blocked),
        passed: Math.round(baseRequests - blocked),
      }
    })
  }, [timeRange])

  return (
    <Card>
      <CardHeader>
        <CardTitle>Traffic Overview</CardTitle>
        <CardDescription>Request volume and blocked threats over time</CardDescription>
      </CardHeader>
      <CardContent>
        <div className="h-[350px]">
          <ResponsiveContainer width="100%" height="100%">
            <AreaChart data={data}>
              <defs>
                <linearGradient id="colorPassed" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%" stopColor="#22c55e" stopOpacity={0.3} />
                  <stop offset="95%" stopColor="#22c55e" stopOpacity={0} />
                </linearGradient>
                <linearGradient id="colorBlocked" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%" stopColor="#ef4444" stopOpacity={0.3} />
                  <stop offset="95%" stopColor="#ef4444" stopOpacity={0} />
                </linearGradient>
              </defs>
              <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
              <XAxis
                dataKey="time"
                tick={{ fontSize: 12 }}
                tickLine={false}
                axisLine={false}
              />
              <YAxis
                tick={{ fontSize: 12 }}
                tickLine={false}
                axisLine={false}
                tickFormatter={(value) => formatNumber(value)}
              />
              <Tooltip
                contentStyle={{
                  backgroundColor: 'hsl(var(--card))',
                  border: '1px solid hsl(var(--border))',
                  borderRadius: '8px',
                }}
                formatter={(value: number) => formatNumber(value)}
              />
              <Legend />
              <Area
                type="monotone"
                dataKey="passed"
                name="Passed"
                stroke="#22c55e"
                fillOpacity={1}
                fill="url(#colorPassed)"
              />
              <Area
                type="monotone"
                dataKey="blocked"
                name="Blocked"
                stroke="#ef4444"
                fillOpacity={1}
                fill="url(#colorBlocked)"
              />
            </AreaChart>
          </ResponsiveContainer>
        </div>
      </CardContent>
    </Card>
  )
}

function LatencyChart({ timeRange }: { timeRange: string }) {
  const data = useMemo(() => {
    const points = timeRange === '1h' ? 12 : timeRange === '24h' ? 24 : 7
    return Array.from({ length: points }, (_, i) => ({
      time: `${i}`,
      p50: 20 + Math.random() * 30,
      p95: 50 + Math.random() * 50,
      p99: 100 + Math.random() * 100,
    }))
  }, [timeRange])

  return (
    <Card>
      <CardHeader>
        <CardTitle>Response Latency</CardTitle>
        <CardDescription>P50, P95, and P99 latency percentiles</CardDescription>
      </CardHeader>
      <CardContent>
        <div className="h-[300px]">
          <ResponsiveContainer width="100%" height="100%">
            <LineChart data={data}>
              <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
              <XAxis dataKey="time" tick={{ fontSize: 12 }} tickLine={false} axisLine={false} />
              <YAxis
                tick={{ fontSize: 12 }}
                tickLine={false}
                axisLine={false}
                tickFormatter={(value) => `${value}ms`}
              />
              <Tooltip
                contentStyle={{
                  backgroundColor: 'hsl(var(--card))',
                  border: '1px solid hsl(var(--border))',
                  borderRadius: '8px',
                }}
                formatter={(value: number) => `${value.toFixed(1)}ms`}
              />
              <Legend />
              <Line type="monotone" dataKey="p50" name="P50" stroke="#22c55e" strokeWidth={2} dot={false} />
              <Line type="monotone" dataKey="p95" name="P95" stroke="#f59e0b" strokeWidth={2} dot={false} />
              <Line type="monotone" dataKey="p99" name="P99" stroke="#ef4444" strokeWidth={2} dot={false} />
            </LineChart>
          </ResponsiveContainer>
        </div>
      </CardContent>
    </Card>
  )
}

function AttackTypesChart() {
  const data = [
    { name: 'SYN Flood', value: 35, count: 12450 },
    { name: 'UDP Flood', value: 25, count: 8900 },
    { name: 'HTTP Flood', value: 20, count: 7100 },
    { name: 'DNS Amplification', value: 12, count: 4260 },
    { name: 'Slowloris', value: 5, count: 1780 },
    { name: 'Other', value: 3, count: 1065 },
  ]

  return (
    <Card>
      <CardHeader>
        <CardTitle>Attack Types</CardTitle>
        <CardDescription>Distribution of blocked attack types</CardDescription>
      </CardHeader>
      <CardContent>
        <div className="h-[300px]">
          <ResponsiveContainer width="100%" height="100%">
            <PieChart>
              <Pie
                data={data}
                cx="50%"
                cy="50%"
                innerRadius={60}
                outerRadius={100}
                paddingAngle={2}
                dataKey="value"
              >
                {data.map((_, index) => (
                  <Cell key={`cell-${index}`} fill={PIE_COLORS[index % PIE_COLORS.length]} />
                ))}
              </Pie>
              <Tooltip
                contentStyle={{
                  backgroundColor: 'hsl(var(--card))',
                  border: '1px solid hsl(var(--border))',
                  borderRadius: '8px',
                }}
                formatter={(value, name, entry) => [
                  `${value}% (${formatNumber((entry.payload as { count: number }).count)} attacks)`,
                  name,
                ]}
              />
              <Legend />
            </PieChart>
          </ResponsiveContainer>
        </div>
      </CardContent>
    </Card>
  )
}

function TopCountriesChart() {
  const data = [
    { country: 'China', code: 'CN', requests: 45230, blocked: 38500 },
    { country: 'Russia', code: 'RU', requests: 32100, blocked: 28900 },
    { country: 'United States', code: 'US', requests: 28500, blocked: 4200 },
    { country: 'Brazil', code: 'BR', requests: 15600, blocked: 8900 },
    { country: 'India', code: 'IN', requests: 12300, blocked: 3100 },
    { country: 'Germany', code: 'DE', requests: 9800, blocked: 1200 },
  ]

  return (
    <Card>
      <CardHeader>
        <CardTitle>Traffic by Country</CardTitle>
        <CardDescription>Top source countries by request volume</CardDescription>
      </CardHeader>
      <CardContent>
        <div className="h-[300px]">
          <ResponsiveContainer width="100%" height="100%">
            <BarChart data={data} layout="vertical">
              <CartesianGrid strokeDasharray="3 3" className="stroke-muted" horizontal={false} />
              <XAxis
                type="number"
                tick={{ fontSize: 12 }}
                tickLine={false}
                axisLine={false}
                tickFormatter={(value) => formatNumber(value)}
              />
              <YAxis
                type="category"
                dataKey="country"
                tick={{ fontSize: 12 }}
                tickLine={false}
                axisLine={false}
                width={100}
              />
              <Tooltip
                contentStyle={{
                  backgroundColor: 'hsl(var(--card))',
                  border: '1px solid hsl(var(--border))',
                  borderRadius: '8px',
                }}
                formatter={(value: number) => formatNumber(value)}
              />
              <Legend />
              <Bar dataKey="requests" name="Total Requests" fill="#3b82f6" radius={[0, 4, 4, 0]} />
              <Bar dataKey="blocked" name="Blocked" fill="#ef4444" radius={[0, 4, 4, 0]} />
            </BarChart>
          </ResponsiveContainer>
        </div>
      </CardContent>
    </Card>
  )
}

function BandwidthChart({ timeRange }: { timeRange: string }) {
  const data = useMemo(() => {
    const points = timeRange === '1h' ? 12 : timeRange === '24h' ? 24 : 7
    return Array.from({ length: points }, (_, i) => ({
      time: `${i}`,
      inbound: (50 + Math.random() * 100) * 1024 * 1024,
      outbound: (30 + Math.random() * 60) * 1024 * 1024,
    }))
  }, [timeRange])

  return (
    <Card>
      <CardHeader>
        <CardTitle>Bandwidth Usage</CardTitle>
        <CardDescription>Inbound and outbound traffic volume</CardDescription>
      </CardHeader>
      <CardContent>
        <div className="h-[300px]">
          <ResponsiveContainer width="100%" height="100%">
            <AreaChart data={data}>
              <defs>
                <linearGradient id="colorInbound" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%" stopColor="#3b82f6" stopOpacity={0.3} />
                  <stop offset="95%" stopColor="#3b82f6" stopOpacity={0} />
                </linearGradient>
                <linearGradient id="colorOutbound" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%" stopColor="#8b5cf6" stopOpacity={0.3} />
                  <stop offset="95%" stopColor="#8b5cf6" stopOpacity={0} />
                </linearGradient>
              </defs>
              <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
              <XAxis dataKey="time" tick={{ fontSize: 12 }} tickLine={false} axisLine={false} />
              <YAxis
                tick={{ fontSize: 12 }}
                tickLine={false}
                axisLine={false}
                tickFormatter={(value) => formatBytes(value)}
              />
              <Tooltip
                contentStyle={{
                  backgroundColor: 'hsl(var(--card))',
                  border: '1px solid hsl(var(--border))',
                  borderRadius: '8px',
                }}
                formatter={(value: number) => formatBytes(value)}
              />
              <Legend />
              <Area
                type="monotone"
                dataKey="inbound"
                name="Inbound"
                stroke="#3b82f6"
                fillOpacity={1}
                fill="url(#colorInbound)"
              />
              <Area
                type="monotone"
                dataKey="outbound"
                name="Outbound"
                stroke="#8b5cf6"
                fillOpacity={1}
                fill="url(#colorOutbound)"
              />
            </AreaChart>
          </ResponsiveContainer>
        </div>
      </CardContent>
    </Card>
  )
}

function StatCard({
  title,
  value,
  change,
  changeType,
  icon: Icon,
}: {
  title: string
  value: string
  change: string
  changeType: 'positive' | 'negative' | 'neutral'
  icon: React.ComponentType<{ className?: string }>
}) {
  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle className="text-sm font-medium">{title}</CardTitle>
        <Icon className="h-4 w-4 text-muted-foreground" />
      </CardHeader>
      <CardContent>
        <div className="text-2xl font-bold">{value}</div>
        <div className="flex items-center gap-1 text-xs">
          {changeType === 'positive' ? (
            <TrendingUp className="h-3 w-3 text-success" />
          ) : changeType === 'negative' ? (
            <TrendingDown className="h-3 w-3 text-destructive" />
          ) : null}
          <span
            className={
              changeType === 'positive'
                ? 'text-success'
                : changeType === 'negative'
                ? 'text-destructive'
                : 'text-muted-foreground'
            }
          >
            {change}
          </span>
          <span className="text-muted-foreground">vs last period</span>
        </div>
      </CardContent>
    </Card>
  )
}

export default function DashboardAnalytics() {
  const [timeRange, setTimeRange] = useState('24h')

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Analytics</h1>
          <p className="text-muted-foreground">
            Detailed insights into your traffic and protection metrics
          </p>
        </div>
        <TimeRangeSelector value={timeRange} onChange={setTimeRange} />
      </div>

      {/* Summary Stats */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <StatCard
          title="Total Requests"
          value="2.4M"
          change="+12.5%"
          changeType="positive"
          icon={Globe}
        />
        <StatCard
          title="Threats Blocked"
          value="156K"
          change="-8.2%"
          changeType="positive"
          icon={Shield}
        />
        <StatCard
          title="Avg Response Time"
          value="45ms"
          change="+2ms"
          changeType="negative"
          icon={TrendingUp}
        />
        <StatCard
          title="Bandwidth"
          value="1.2 TB"
          change="+18%"
          changeType="neutral"
          icon={BarChart3}
        />
      </div>

      {/* Main Traffic Chart */}
      <TrafficChart timeRange={timeRange} />

      {/* Secondary Charts */}
      <Tabs defaultValue="latency" className="space-y-4">
        <TabsList>
          <TabsTrigger value="latency">Latency</TabsTrigger>
          <TabsTrigger value="bandwidth">Bandwidth</TabsTrigger>
          <TabsTrigger value="attacks">Attack Types</TabsTrigger>
          <TabsTrigger value="geography">Geography</TabsTrigger>
        </TabsList>

        <TabsContent value="latency" className="space-y-4">
          <LatencyChart timeRange={timeRange} />
        </TabsContent>

        <TabsContent value="bandwidth" className="space-y-4">
          <BandwidthChart timeRange={timeRange} />
        </TabsContent>

        <TabsContent value="attacks" className="space-y-4">
          <AttackTypesChart />
        </TabsContent>

        <TabsContent value="geography" className="space-y-4">
          <TopCountriesChart />
        </TabsContent>
      </Tabs>

      {/* Bottom Row */}
      <div className="grid gap-4 md:grid-cols-2">
        <AttackTypesChart />
        <TopCountriesChart />
      </div>
    </div>
  )
}
