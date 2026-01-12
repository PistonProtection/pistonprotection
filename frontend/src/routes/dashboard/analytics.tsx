import { useState, useMemo } from "react"
import { createFileRoute } from "@tanstack/react-router"
import { useQuery } from "@tanstack/react-query"
import {
  Area,
  AreaChart,
  Bar,
  BarChart,
  CartesianGrid,
  Line,
  LineChart,
  ResponsiveContainer,
  XAxis,
  YAxis,
  Tooltip as RechartsTooltip,
  Legend,
  PieChart,
  Pie,
  Cell,
} from "recharts"
import { Calendar, Download, RefreshCw } from "lucide-react"
import { format, subDays, subHours } from "date-fns"

import { cn } from "@/lib/utils"
import { analyticsOptions } from "@/lib/api"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Skeleton } from "@/components/ui/skeleton"

export const Route = createFileRoute("/dashboard/analytics")({
  component: AnalyticsPage,
})

type TimeRange = "1h" | "24h" | "7d" | "30d"
type Granularity = "minute" | "hour" | "day"

function getTimeParams(range: TimeRange): {
  startDate: string
  endDate: string
  granularity: Granularity
} {
  const now = new Date()
  let startDate: Date
  let granularity: Granularity

  switch (range) {
    case "1h":
      startDate = subHours(now, 1)
      granularity = "minute"
      break
    case "24h":
      startDate = subHours(now, 24)
      granularity = "hour"
      break
    case "7d":
      startDate = subDays(now, 7)
      granularity = "hour"
      break
    case "30d":
      startDate = subDays(now, 30)
      granularity = "day"
      break
  }

  return {
    startDate: startDate.toISOString(),
    endDate: now.toISOString(),
    granularity,
  }
}

function formatNumber(num: number): string {
  if (num >= 1_000_000) {
    return (num / 1_000_000).toFixed(1) + "M"
  }
  if (num >= 1_000) {
    return (num / 1_000).toFixed(1) + "K"
  }
  return num.toFixed(0)
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

const COLORS = {
  requests: "hsl(var(--chart-1))",
  blocked: "hsl(var(--destructive))",
  allowed: "hsl(var(--chart-2))",
  challenged: "hsl(var(--chart-4))",
  latencyP50: "hsl(var(--chart-1))",
  latencyP95: "hsl(var(--chart-3))",
  latencyP99: "hsl(var(--chart-5))",
}

const PIE_COLORS = [
  "hsl(var(--chart-2))",
  "hsl(var(--destructive))",
  "hsl(var(--chart-4))",
]

function AnalyticsPage() {
  const [timeRange, setTimeRange] = useState<TimeRange>("24h")
  const timeParams = useMemo(() => getTimeParams(timeRange), [timeRange])

  const {
    data: analyticsData,
    isLoading,
    refetch,
  } = useQuery(analyticsOptions(timeParams))

  const chartData = useMemo(() => {
    if (!analyticsData) return []
    return analyticsData.map((item) => ({
      ...item,
      time: format(
        new Date(item.timestamp),
        timeRange === "1h"
          ? "HH:mm"
          : timeRange === "24h"
            ? "HH:mm"
            : timeRange === "7d"
              ? "MM/dd HH:mm"
              : "MM/dd"
      ),
    }))
  }, [analyticsData, timeRange])

  const totals = useMemo(() => {
    if (!analyticsData)
      return {
        requests: 0,
        blocked: 0,
        allowed: 0,
        challenged: 0,
        bandwidth: 0,
      }
    return analyticsData.reduce(
      (acc, item) => ({
        requests: acc.requests + item.requests,
        blocked: acc.blocked + item.blocked,
        allowed: acc.allowed + item.allowed,
        challenged: acc.challenged + item.challenged,
        bandwidth: acc.bandwidth + item.bandwidth,
      }),
      { requests: 0, blocked: 0, allowed: 0, challenged: 0, bandwidth: 0 }
    )
  }, [analyticsData])

  const pieData = useMemo(() => {
    return [
      { name: "Allowed", value: totals.allowed },
      { name: "Blocked", value: totals.blocked },
      { name: "Challenged", value: totals.challenged },
    ]
  }, [totals])

  return (
    <div className="flex-1 space-y-6 p-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Analytics</h1>
          <p className="text-muted-foreground">
            Detailed traffic analysis and performance metrics.
          </p>
        </div>
        <div className="flex items-center gap-2">
          <Select
            value={timeRange}
            onValueChange={(value: TimeRange) => setTimeRange(value)}
          >
            <SelectTrigger className="w-36">
              <Calendar className="mr-2 size-4" />
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="1h">Last Hour</SelectItem>
              <SelectItem value="24h">Last 24 Hours</SelectItem>
              <SelectItem value="7d">Last 7 Days</SelectItem>
              <SelectItem value="30d">Last 30 Days</SelectItem>
            </SelectContent>
          </Select>
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
          <Button variant="outline" size="sm">
            <Download className="mr-2 size-4" />
            Export
          </Button>
        </div>
      </div>

      {/* Summary Stats */}
      <div className="grid gap-4 md:grid-cols-4">
        <Card>
          <CardHeader className="pb-2">
            <CardDescription>Total Requests</CardDescription>
          </CardHeader>
          <CardContent>
            {isLoading ? (
              <Skeleton className="h-8 w-24" />
            ) : (
              <div className="text-2xl font-bold">
                {formatNumber(totals.requests)}
              </div>
            )}
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardDescription>Blocked</CardDescription>
          </CardHeader>
          <CardContent>
            {isLoading ? (
              <Skeleton className="h-8 w-24" />
            ) : (
              <div className="text-2xl font-bold text-destructive">
                {formatNumber(totals.blocked)}
              </div>
            )}
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardDescription>Challenged</CardDescription>
          </CardHeader>
          <CardContent>
            {isLoading ? (
              <Skeleton className="h-8 w-24" />
            ) : (
              <div className="text-2xl font-bold text-yellow-600 dark:text-yellow-400">
                {formatNumber(totals.challenged)}
              </div>
            )}
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardDescription>Bandwidth</CardDescription>
          </CardHeader>
          <CardContent>
            {isLoading ? (
              <Skeleton className="h-8 w-24" />
            ) : (
              <div className="text-2xl font-bold">
                {formatBytes(totals.bandwidth)}
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      {/* Charts */}
      <Tabs defaultValue="traffic" className="space-y-4">
        <TabsList>
          <TabsTrigger value="traffic">Traffic</TabsTrigger>
          <TabsTrigger value="security">Security</TabsTrigger>
          <TabsTrigger value="latency">Latency</TabsTrigger>
          <TabsTrigger value="distribution">Distribution</TabsTrigger>
        </TabsList>

        <TabsContent value="traffic" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Request Volume</CardTitle>
              <CardDescription>
                Total requests over the selected time period
              </CardDescription>
            </CardHeader>
            <CardContent>
              {isLoading ? (
                <Skeleton className="h-[400px] w-full" />
              ) : (
                <div className="h-[400px]">
                  <ResponsiveContainer width="100%" height="100%">
                    <AreaChart data={chartData}>
                      <defs>
                        <linearGradient id="colorRequests" x1="0" y1="0" x2="0" y2="1">
                          <stop
                            offset="5%"
                            stopColor={COLORS.requests}
                            stopOpacity={0.3}
                          />
                          <stop
                            offset="95%"
                            stopColor={COLORS.requests}
                            stopOpacity={0}
                          />
                        </linearGradient>
                      </defs>
                      <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
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
                        formatter={(value: number) => [formatNumber(value), "Requests"]}
                      />
                      <Area
                        type="monotone"
                        dataKey="requests"
                        stroke={COLORS.requests}
                        fill="url(#colorRequests)"
                        strokeWidth={2}
                      />
                    </AreaChart>
                  </ResponsiveContainer>
                </div>
              )}
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Bandwidth Usage</CardTitle>
              <CardDescription>
                Data transfer over the selected time period
              </CardDescription>
            </CardHeader>
            <CardContent>
              {isLoading ? (
                <Skeleton className="h-[300px] w-full" />
              ) : (
                <div className="h-[300px]">
                  <ResponsiveContainer width="100%" height="100%">
                    <BarChart data={chartData}>
                      <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
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
                        tickFormatter={(value) => formatBytes(value)}
                      />
                      <RechartsTooltip
                        contentStyle={{
                          backgroundColor: "hsl(var(--popover))",
                          border: "1px solid hsl(var(--border))",
                          borderRadius: "8px",
                          color: "hsl(var(--popover-foreground))",
                        }}
                        formatter={(value: number) => [formatBytes(value), "Bandwidth"]}
                      />
                      <Bar
                        dataKey="bandwidth"
                        fill={COLORS.requests}
                        radius={[4, 4, 0, 0]}
                      />
                    </BarChart>
                  </ResponsiveContainer>
                </div>
              )}
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="security" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Traffic Classification</CardTitle>
              <CardDescription>
                Breakdown of allowed, blocked, and challenged requests
              </CardDescription>
            </CardHeader>
            <CardContent>
              {isLoading ? (
                <Skeleton className="h-[400px] w-full" />
              ) : (
                <div className="h-[400px]">
                  <ResponsiveContainer width="100%" height="100%">
                    <AreaChart data={chartData}>
                      <defs>
                        <linearGradient id="colorAllowed" x1="0" y1="0" x2="0" y2="1">
                          <stop
                            offset="5%"
                            stopColor={COLORS.allowed}
                            stopOpacity={0.3}
                          />
                          <stop
                            offset="95%"
                            stopColor={COLORS.allowed}
                            stopOpacity={0}
                          />
                        </linearGradient>
                        <linearGradient id="colorBlocked" x1="0" y1="0" x2="0" y2="1">
                          <stop
                            offset="5%"
                            stopColor={COLORS.blocked}
                            stopOpacity={0.3}
                          />
                          <stop
                            offset="95%"
                            stopColor={COLORS.blocked}
                            stopOpacity={0}
                          />
                        </linearGradient>
                        <linearGradient id="colorChallenged" x1="0" y1="0" x2="0" y2="1">
                          <stop
                            offset="5%"
                            stopColor={COLORS.challenged}
                            stopOpacity={0.3}
                          />
                          <stop
                            offset="95%"
                            stopColor={COLORS.challenged}
                            stopOpacity={0}
                          />
                        </linearGradient>
                      </defs>
                      <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
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
                        formatter={(value: number) => formatNumber(value)}
                      />
                      <Legend />
                      <Area
                        type="monotone"
                        dataKey="allowed"
                        name="Allowed"
                        stroke={COLORS.allowed}
                        fill="url(#colorAllowed)"
                        strokeWidth={2}
                        stackId="1"
                      />
                      <Area
                        type="monotone"
                        dataKey="blocked"
                        name="Blocked"
                        stroke={COLORS.blocked}
                        fill="url(#colorBlocked)"
                        strokeWidth={2}
                        stackId="1"
                      />
                      <Area
                        type="monotone"
                        dataKey="challenged"
                        name="Challenged"
                        stroke={COLORS.challenged}
                        fill="url(#colorChallenged)"
                        strokeWidth={2}
                        stackId="1"
                      />
                    </AreaChart>
                  </ResponsiveContainer>
                </div>
              )}
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="latency" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Latency Percentiles</CardTitle>
              <CardDescription>
                Response time distribution (P50, P95, P99)
              </CardDescription>
            </CardHeader>
            <CardContent>
              {isLoading ? (
                <Skeleton className="h-[400px] w-full" />
              ) : (
                <div className="h-[400px]">
                  <ResponsiveContainer width="100%" height="100%">
                    <LineChart data={chartData}>
                      <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
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
                        tickFormatter={(value) => `${value}ms`}
                      />
                      <RechartsTooltip
                        contentStyle={{
                          backgroundColor: "hsl(var(--popover))",
                          border: "1px solid hsl(var(--border))",
                          borderRadius: "8px",
                          color: "hsl(var(--popover-foreground))",
                        }}
                        formatter={(value: number) => [`${value.toFixed(1)}ms`]}
                      />
                      <Legend />
                      <Line
                        type="monotone"
                        dataKey="latencyP50"
                        name="P50"
                        stroke={COLORS.latencyP50}
                        strokeWidth={2}
                        dot={false}
                      />
                      <Line
                        type="monotone"
                        dataKey="latencyP95"
                        name="P95"
                        stroke={COLORS.latencyP95}
                        strokeWidth={2}
                        dot={false}
                      />
                      <Line
                        type="monotone"
                        dataKey="latencyP99"
                        name="P99"
                        stroke={COLORS.latencyP99}
                        strokeWidth={2}
                        dot={false}
                      />
                    </LineChart>
                  </ResponsiveContainer>
                </div>
              )}
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="distribution" className="space-y-4">
          <div className="grid gap-4 md:grid-cols-2">
            <Card>
              <CardHeader>
                <CardTitle>Request Distribution</CardTitle>
                <CardDescription>
                  Breakdown of traffic by classification
                </CardDescription>
              </CardHeader>
              <CardContent>
                {isLoading ? (
                  <Skeleton className="h-[300px] w-full" />
                ) : (
                  <div className="h-[300px]">
                    <ResponsiveContainer width="100%" height="100%">
                      <PieChart>
                        <Pie
                          data={pieData}
                          cx="50%"
                          cy="50%"
                          innerRadius={60}
                          outerRadius={100}
                          paddingAngle={2}
                          dataKey="value"
                          label={({ name, percent }) =>
                            `${name} ${(percent * 100).toFixed(1)}%`
                          }
                          labelLine={false}
                        >
                          {pieData.map((_, index) => (
                            <Cell
                              key={`cell-${index}`}
                              fill={PIE_COLORS[index % PIE_COLORS.length]}
                            />
                          ))}
                        </Pie>
                        <RechartsTooltip
                          contentStyle={{
                            backgroundColor: "hsl(var(--popover))",
                            border: "1px solid hsl(var(--border))",
                            borderRadius: "8px",
                            color: "hsl(var(--popover-foreground))",
                          }}
                          formatter={(value: number) => formatNumber(value)}
                        />
                      </PieChart>
                    </ResponsiveContainer>
                  </div>
                )}
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>Traffic Summary</CardTitle>
                <CardDescription>
                  Detailed breakdown of traffic metrics
                </CardDescription>
              </CardHeader>
              <CardContent>
                {isLoading ? (
                  <div className="space-y-4">
                    {[1, 2, 3, 4].map((i) => (
                      <Skeleton key={i} className="h-12 w-full" />
                    ))}
                  </div>
                ) : (
                  <div className="space-y-4">
                    <div className="flex items-center justify-between p-3 rounded-lg bg-muted/50">
                      <div className="flex items-center gap-3">
                        <div className="size-3 rounded-full bg-[hsl(var(--chart-2))]" />
                        <span className="font-medium">Allowed</span>
                      </div>
                      <div className="text-right">
                        <p className="font-bold">{formatNumber(totals.allowed)}</p>
                        <p className="text-xs text-muted-foreground">
                          {totals.requests > 0
                            ? ((totals.allowed / totals.requests) * 100).toFixed(1)
                            : 0}
                          %
                        </p>
                      </div>
                    </div>

                    <div className="flex items-center justify-between p-3 rounded-lg bg-muted/50">
                      <div className="flex items-center gap-3">
                        <div className="size-3 rounded-full bg-destructive" />
                        <span className="font-medium">Blocked</span>
                      </div>
                      <div className="text-right">
                        <p className="font-bold text-destructive">
                          {formatNumber(totals.blocked)}
                        </p>
                        <p className="text-xs text-muted-foreground">
                          {totals.requests > 0
                            ? ((totals.blocked / totals.requests) * 100).toFixed(1)
                            : 0}
                          %
                        </p>
                      </div>
                    </div>

                    <div className="flex items-center justify-between p-3 rounded-lg bg-muted/50">
                      <div className="flex items-center gap-3">
                        <div className="size-3 rounded-full bg-[hsl(var(--chart-4))]" />
                        <span className="font-medium">Challenged</span>
                      </div>
                      <div className="text-right">
                        <p className="font-bold text-yellow-600 dark:text-yellow-400">
                          {formatNumber(totals.challenged)}
                        </p>
                        <p className="text-xs text-muted-foreground">
                          {totals.requests > 0
                            ? ((totals.challenged / totals.requests) * 100).toFixed(
                                1
                              )
                            : 0}
                          %
                        </p>
                      </div>
                    </div>

                    <div className="flex items-center justify-between p-3 rounded-lg bg-muted/50">
                      <div className="flex items-center gap-3">
                        <div className="size-3 rounded-full bg-primary" />
                        <span className="font-medium">Total</span>
                      </div>
                      <div className="text-right">
                        <p className="font-bold">{formatNumber(totals.requests)}</p>
                        <p className="text-xs text-muted-foreground">
                          {formatBytes(totals.bandwidth)} bandwidth
                        </p>
                      </div>
                    </div>
                  </div>
                )}
              </CardContent>
            </Card>
          </div>
        </TabsContent>
      </Tabs>
    </div>
  )
}
