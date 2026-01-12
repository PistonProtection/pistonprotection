import { createFileRoute } from "@tanstack/react-router"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Activity, Globe, Shield, TrendingUp, ArrowUpRight, ArrowDownRight, Zap, Server } from "lucide-react"

export const Route = createFileRoute("/dashboard/analytics")({ component: AnalyticsPage })

function AnalyticsPage() {
  const trafficData = [
    { time: "00:00", requests: 12400, blocked: 234 },
    { time: "04:00", requests: 8900, blocked: 156 },
    { time: "08:00", requests: 23400, blocked: 567 },
    { time: "12:00", requests: 45200, blocked: 1234 },
    { time: "16:00", requests: 38700, blocked: 892 },
    { time: "20:00", requests: 29800, blocked: 445 },
  ]
  const topAttackSources = [
    { country: "Russia", attacks: 4521, percentage: 28 },
    { country: "China", attacks: 3892, percentage: 24 },
    { country: "United States", attacks: 2145, percentage: 13 },
    { country: "Brazil", attacks: 1876, percentage: 12 },
    { country: "India", attacks: 1234, percentage: 8 },
  ]
  const attackTypes = [
    { type: "SYN Flood", count: 5234, trend: "up", change: 12 },
    { type: "UDP Amplification", count: 3456, trend: "down", change: 8 },
    { type: "HTTP Flood", count: 2891, trend: "up", change: 23 },
    { type: "DNS Amplification", count: 1234, trend: "down", change: 15 },
    { type: "Slowloris", count: 567, trend: "up", change: 5 },
  ]
  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div><h1 className="text-2xl font-bold tracking-tight">Analytics</h1><p className="text-muted-foreground">Traffic analysis and threat intelligence.</p></div>
        <Select defaultValue="24h"><SelectTrigger className="w-[180px]"><SelectValue placeholder="Select range" /></SelectTrigger><SelectContent><SelectItem value="1h">Last Hour</SelectItem><SelectItem value="24h">Last 24 Hours</SelectItem><SelectItem value="7d">Last 7 Days</SelectItem><SelectItem value="30d">Last 30 Days</SelectItem></SelectContent></Select>
      </div>
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <Card><CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2"><CardTitle className="text-sm font-medium">Total Requests</CardTitle><Activity className="h-4 w-4 text-muted-foreground" /></CardHeader><CardContent><div className="text-2xl font-bold">158.4M</div><p className="text-xs text-muted-foreground"><span className="text-green-500 inline-flex items-center"><ArrowUpRight className="h-3 w-3" /> +12.5%</span> from last period</p></CardContent></Card>
        <Card><CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2"><CardTitle className="text-sm font-medium">Blocked Attacks</CardTitle><Shield className="h-4 w-4 text-red-500" /></CardHeader><CardContent><div className="text-2xl font-bold">16,382</div><p className="text-xs text-muted-foreground"><span className="text-red-500 inline-flex items-center"><ArrowUpRight className="h-3 w-3" /> +28.3%</span> from last period</p></CardContent></Card>
        <Card><CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2"><CardTitle className="text-sm font-medium">Peak Traffic</CardTitle><TrendingUp className="h-4 w-4 text-muted-foreground" /></CardHeader><CardContent><div className="text-2xl font-bold">89.2K/s</div><p className="text-xs text-muted-foreground">Recorded at 14:32 UTC</p></CardContent></Card>
        <Card><CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2"><CardTitle className="text-sm font-medium">Uptime</CardTitle><Server className="h-4 w-4 text-green-500" /></CardHeader><CardContent><div className="text-2xl font-bold">99.99%</div><p className="text-xs text-muted-foreground">Last 30 days</p></CardContent></Card>
      </div>
      <div className="grid gap-4 md:grid-cols-2">
        <Card>
          <CardHeader><CardTitle>Traffic Overview</CardTitle><CardDescription>Requests and blocked attacks over time.</CardDescription></CardHeader>
          <CardContent>
            <div className="space-y-4">
              {trafficData.map((d, i) => (
                <div key={i} className="flex items-center gap-4">
                  <span className="w-14 text-sm text-muted-foreground">{d.time}</span>
                  <div className="flex-1 space-y-1">
                    <div className="flex items-center gap-2"><div className="h-2 bg-primary rounded" style={{ width: `${(d.requests / 50000) * 100}%` }} /><span className="text-xs">{(d.requests / 1000).toFixed(1)}K</span></div>
                    <div className="flex items-center gap-2"><div className="h-2 bg-destructive rounded" style={{ width: `${(d.blocked / 1500) * 100}%` }} /><span className="text-xs text-destructive">{d.blocked}</span></div>
                  </div>
                </div>
              ))}
            </div>
            <div className="flex items-center gap-4 mt-4 text-sm"><div className="flex items-center gap-2"><div className="h-3 w-3 rounded bg-primary" /><span>Requests</span></div><div className="flex items-center gap-2"><div className="h-3 w-3 rounded bg-destructive" /><span>Blocked</span></div></div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader><CardTitle>Top Attack Sources</CardTitle><CardDescription>Geographic distribution of attacks.</CardDescription></CardHeader>
          <CardContent>
            <div className="space-y-4">
              {topAttackSources.map((s, i) => (
                <div key={i} className="flex items-center justify-between">
                  <div className="flex items-center gap-3"><Globe className="h-4 w-4 text-muted-foreground" /><span className="font-medium">{s.country}</span></div>
                  <div className="flex items-center gap-4"><div className="w-32 h-2 bg-muted rounded"><div className="h-2 bg-destructive rounded" style={{ width: `${s.percentage}%` }} /></div><span className="text-sm text-muted-foreground w-16 text-right">{s.attacks.toLocaleString()}</span></div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      </div>
      <Card>
        <CardHeader><CardTitle>Attack Type Analysis</CardTitle><CardDescription>Breakdown of detected attack vectors.</CardDescription></CardHeader>
        <CardContent>
          <Tabs defaultValue="all">
            <TabsList><TabsTrigger value="all">All Types</TabsTrigger><TabsTrigger value="network">Network (L3/L4)</TabsTrigger><TabsTrigger value="application">Application (L7)</TabsTrigger></TabsList>
            <TabsContent value="all" className="mt-4">
              <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-5">
                {attackTypes.map((a, i) => (
                  <Card key={i}>
                    <CardContent className="pt-6">
                      <div className="flex items-center justify-between mb-2"><Zap className="h-4 w-4 text-muted-foreground" /><Badge variant={a.trend === "up" ? "destructive" : "secondary"}>{a.trend === "up" ? <ArrowUpRight className="h-3 w-3 mr-1" /> : <ArrowDownRight className="h-3 w-3 mr-1" />}{a.change}%</Badge></div>
                      <div className="text-2xl font-bold">{a.count.toLocaleString()}</div>
                      <p className="text-xs text-muted-foreground">{a.type}</p>
                    </CardContent>
                  </Card>
                ))}
              </div>
            </TabsContent>
            <TabsContent value="network" className="mt-4"><p className="text-muted-foreground">Network layer attack statistics will appear here.</p></TabsContent>
            <TabsContent value="application" className="mt-4"><p className="text-muted-foreground">Application layer attack statistics will appear here.</p></TabsContent>
          </Tabs>
        </CardContent>
      </Card>
    </div>
  )
}
