import { createFileRoute } from "@tanstack/react-router"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import { ChartContainer, ChartTooltip, ChartTooltipContent } from "@/components/ui/chart"
import { Area, AreaChart, Bar, BarChart, CartesianGrid, XAxis, YAxis, ResponsiveContainer } from "recharts"
import { Activity, Shield, Globe, AlertTriangle, TrendingUp, TrendingDown } from "lucide-react"

export const Route = createFileRoute("/dashboard/analytics")({ component: AnalyticsPage })

const trafficData = [
  { time: "00:00", requests: 42000, blocked: 1200, allowed: 40800 },
  { time: "04:00", requests: 31000, blocked: 750, allowed: 30250 },
  { time: "08:00", requests: 52000, blocked: 2100, allowed: 49900 },
  { time: "12:00", requests: 72000, blocked: 4200, allowed: 67800 },
  { time: "16:00", requests: 82000, blocked: 4800, allowed: 77200 },
  { time: "20:00", requests: 64000, blocked: 2800, allowed: 61200 },
]

const attackTypeData = [
  { type: "SYN Flood", count: 12450 },
  { type: "UDP Amplification", count: 8920 },
  { type: "HTTP Flood", count: 6780 },
  { type: "DNS Amplification", count: 4560 },
  { type: "QUIC Flood", count: 2340 },
]

const topAttackers = [
  { ip: "185.220.101.x", country: "RU", attacks: 4521, status: "blocked" },
  { ip: "45.33.32.x", country: "US", attacks: 3892, status: "rate_limited" },
  { ip: "203.0.113.x", country: "CN", attacks: 2987, status: "blocked" },
  { ip: "198.51.100.x", country: "NL", attacks: 2456, status: "blocked" },
  { ip: "192.0.2.x", country: "DE", attacks: 1987, status: "challenged" },
]

const chartConfig = {
  requests: { label: "Total Requests", color: "hsl(var(--chart-1))" },
  blocked: { label: "Blocked", color: "hsl(var(--chart-5))" },
  allowed: { label: "Allowed", color: "hsl(var(--chart-2))" },
}

function AnalyticsPage() {
  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div><h1 className="text-2xl font-bold tracking-tight">Analytics</h1><p className="text-muted-foreground">Monitor traffic patterns and attack statistics.</p></div>
        <Select defaultValue="24h"><SelectTrigger className="w-[180px]"><SelectValue placeholder="Select timeframe" /></SelectTrigger><SelectContent><SelectItem value="1h">Last hour</SelectItem><SelectItem value="6h">Last 6 hours</SelectItem><SelectItem value="24h">Last 24 hours</SelectItem><SelectItem value="7d">Last 7 days</SelectItem></SelectContent></Select>
      </div>

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <Card><CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2"><CardTitle className="text-sm font-medium">Total Requests</CardTitle><Activity className="h-4 w-4 text-muted-foreground" /></CardHeader><CardContent><div className="text-2xl font-bold">686K</div><p className="text-xs text-muted-foreground flex items-center gap-1"><TrendingUp className="h-3 w-3 text-green-500" /><span className="text-green-500">+12.5%</span> from yesterday</p></CardContent></Card>
        <Card><CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2"><CardTitle className="text-sm font-medium">Blocked Requests</CardTitle><Shield className="h-4 w-4 text-red-500" /></CardHeader><CardContent><div className="text-2xl font-bold">31.7K</div><p className="text-xs text-muted-foreground flex items-center gap-1"><TrendingDown className="h-3 w-3 text-green-500" /><span className="text-green-500">-8.2%</span> from yesterday</p></CardContent></Card>
        <Card><CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2"><CardTitle className="text-sm font-medium">Attack Sources</CardTitle><Globe className="h-4 w-4 text-muted-foreground" /></CardHeader><CardContent><div className="text-2xl font-bold">1,284</div><p className="text-xs text-muted-foreground">Unique IPs blocked</p></CardContent></Card>
        <Card><CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2"><CardTitle className="text-sm font-medium">Block Rate</CardTitle><AlertTriangle className="h-4 w-4 text-yellow-500" /></CardHeader><CardContent><div className="text-2xl font-bold">4.62%</div><p className="text-xs text-muted-foreground">Of total traffic</p></CardContent></Card>
      </div>

      <Card>
        <CardHeader><CardTitle>Traffic Overview</CardTitle><CardDescription>Request volume over the last 24 hours.</CardDescription></CardHeader>
        <CardContent>
          <ChartContainer config={chartConfig} className="h-[300px] w-full">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={trafficData}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="time" />
                <YAxis />
                <ChartTooltip content={<ChartTooltipContent />} />
                <Area type="monotone" dataKey="allowed" stackId="1" stroke="var(--color-allowed)" fill="var(--color-allowed)" fillOpacity={0.6} />
                <Area type="monotone" dataKey="blocked" stackId="1" stroke="var(--color-blocked)" fill="var(--color-blocked)" fillOpacity={0.6} />
              </AreaChart>
            </ResponsiveContainer>
          </ChartContainer>
        </CardContent>
      </Card>

      <div className="grid gap-4 md:grid-cols-2">
        <Card>
          <CardHeader><CardTitle>Attack Types</CardTitle><CardDescription>Distribution of blocked attacks by type.</CardDescription></CardHeader>
          <CardContent>
            <ChartContainer config={chartConfig} className="h-[250px] w-full">
              <ResponsiveContainer width="100%" height="100%">
                <BarChart data={attackTypeData} layout="vertical">
                  <CartesianGrid strokeDasharray="3 3" />
                  <XAxis type="number" />
                  <YAxis dataKey="type" type="category" width={120} />
                  <ChartTooltip content={<ChartTooltipContent />} />
                  <Bar dataKey="count" fill="var(--color-blocked)" radius={4} />
                </BarChart>
              </ResponsiveContainer>
            </ChartContainer>
          </CardContent>
        </Card>

        <Card>
          <CardHeader><CardTitle>Top Attack Sources</CardTitle><CardDescription>Most active malicious IP addresses.</CardDescription></CardHeader>
          <CardContent>
            <Table>
              <TableHeader><TableRow><TableHead>IP Address</TableHead><TableHead>Country</TableHead><TableHead className="text-right">Attacks</TableHead><TableHead>Status</TableHead></TableRow></TableHeader>
              <TableBody>
                {topAttackers.map((attacker, i) => (
                  <TableRow key={i}>
                    <TableCell className="font-mono text-sm">{attacker.ip}</TableCell>
                    <TableCell>{attacker.country}</TableCell>
                    <TableCell className="text-right">{attacker.attacks.toLocaleString()}</TableCell>
                    <TableCell><Badge variant={attacker.status === "blocked" ? "destructive" : attacker.status === "rate_limited" ? "secondary" : "outline"}>{attacker.status.replace("_", " ")}</Badge></TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader><CardTitle>Protocol Analysis</CardTitle><CardDescription>Traffic breakdown by protocol type.</CardDescription></CardHeader>
        <CardContent>
          <Tabs defaultValue="tcp">
            <TabsList><TabsTrigger value="tcp">TCP</TabsTrigger><TabsTrigger value="udp">UDP</TabsTrigger><TabsTrigger value="http">HTTP</TabsTrigger><TabsTrigger value="quic">QUIC</TabsTrigger></TabsList>
            <TabsContent value="tcp" className="space-y-4">
              <div className="grid gap-4 md:grid-cols-3 mt-4">
                <Card><CardHeader className="pb-2"><CardTitle className="text-sm">Total Connections</CardTitle></CardHeader><CardContent><div className="text-xl font-bold">245.8K</div></CardContent></Card>
                <Card><CardHeader className="pb-2"><CardTitle className="text-sm">SYN Floods Blocked</CardTitle></CardHeader><CardContent><div className="text-xl font-bold text-red-500">12.4K</div></CardContent></Card>
                <Card><CardHeader className="pb-2"><CardTitle className="text-sm">ACK Floods Blocked</CardTitle></CardHeader><CardContent><div className="text-xl font-bold text-red-500">3.2K</div></CardContent></Card>
              </div>
            </TabsContent>
            <TabsContent value="udp" className="space-y-4">
              <div className="grid gap-4 md:grid-cols-3 mt-4">
                <Card><CardHeader className="pb-2"><CardTitle className="text-sm">Total Packets</CardTitle></CardHeader><CardContent><div className="text-xl font-bold">892.3K</div></CardContent></Card>
                <Card><CardHeader className="pb-2"><CardTitle className="text-sm">Amplification Blocked</CardTitle></CardHeader><CardContent><div className="text-xl font-bold text-red-500">8.9K</div></CardContent></Card>
                <Card><CardHeader className="pb-2"><CardTitle className="text-sm">DNS Attacks Blocked</CardTitle></CardHeader><CardContent><div className="text-xl font-bold text-red-500">4.5K</div></CardContent></Card>
              </div>
            </TabsContent>
            <TabsContent value="http" className="space-y-4">
              <div className="grid gap-4 md:grid-cols-3 mt-4">
                <Card><CardHeader className="pb-2"><CardTitle className="text-sm">Total Requests</CardTitle></CardHeader><CardContent><div className="text-xl font-bold">156.7K</div></CardContent></Card>
                <Card><CardHeader className="pb-2"><CardTitle className="text-sm">L7 Attacks Blocked</CardTitle></CardHeader><CardContent><div className="text-xl font-bold text-red-500">6.7K</div></CardContent></Card>
                <Card><CardHeader className="pb-2"><CardTitle className="text-sm">Challenges Issued</CardTitle></CardHeader><CardContent><div className="text-xl font-bold text-yellow-500">9.8K</div></CardContent></Card>
              </div>
            </TabsContent>
            <TabsContent value="quic" className="space-y-4">
              <div className="grid gap-4 md:grid-cols-3 mt-4">
                <Card><CardHeader className="pb-2"><CardTitle className="text-sm">Total Connections</CardTitle></CardHeader><CardContent><div className="text-xl font-bold">34.2K</div></CardContent></Card>
                <Card><CardHeader className="pb-2"><CardTitle className="text-sm">Invalid Packets</CardTitle></CardHeader><CardContent><div className="text-xl font-bold text-red-500">2.3K</div></CardContent></Card>
                <Card><CardHeader className="pb-2"><CardTitle className="text-sm">Retry Floods</CardTitle></CardHeader><CardContent><div className="text-xl font-bold text-red-500">890</div></CardContent></Card>
              </div>
            </TabsContent>
          </Tabs>
        </CardContent>
      </Card>
    </div>
  )
}
