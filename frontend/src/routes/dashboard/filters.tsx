import { createFileRoute } from "@tanstack/react-router"
import { useState } from "react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Switch } from "@/components/ui/switch"
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "@/components/ui/dropdown-menu"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Filter, Plus, MoreVertical, Pencil, Trash2, Shield, Zap, Globe, Server } from "lucide-react"

export const Route = createFileRoute("/dashboard/filters")({ component: FiltersPage })

interface FilterRule {
  id: string; name: string; type: "tcp" | "udp" | "http" | "quic"; action: "allow" | "block" | "rate_limit" | "challenge"; enabled: boolean; priority: number; conditions: string; hits: number
}

const mockFilters: FilterRule[] = [
  { id: "1", name: "Block SYN Flood", type: "tcp", action: "block", enabled: true, priority: 100, conditions: "SYN rate > 10000/s per IP", hits: 15234 },
  { id: "2", name: "Rate Limit UDP", type: "udp", action: "rate_limit", enabled: true, priority: 90, conditions: "UDP packets > 5000/s per IP", hits: 8721 },
  { id: "3", name: "Block DNS Amplification", type: "udp", action: "block", enabled: true, priority: 95, conditions: "Port 53, payload > 512 bytes", hits: 4532 },
  { id: "4", name: "HTTP Request Limit", type: "http", action: "rate_limit", enabled: true, priority: 80, conditions: "Requests > 100/s per IP", hits: 23456 },
  { id: "5", name: "Challenge Suspicious UA", type: "http", action: "challenge", enabled: true, priority: 70, conditions: "User-Agent matches bot patterns", hits: 9876 },
  { id: "6", name: "Block Invalid QUIC", type: "quic", action: "block", enabled: true, priority: 85, conditions: "Invalid QUIC initial packet", hits: 2345 },
  { id: "7", name: "Allow Trusted IPs", type: "tcp", action: "allow", enabled: true, priority: 200, conditions: "Source IP in allowlist", hits: 543210 },
  { id: "8", name: "Block NTP Amplification", type: "udp", action: "block", enabled: false, priority: 95, conditions: "Port 123, monlist command", hits: 1234 },
]

function FiltersPage() {
  const [filters, setFilters] = useState<FilterRule[]>(mockFilters)
  const [isAddDialogOpen, setIsAddDialogOpen] = useState(false)
  const [activeTab, setActiveTab] = useState("all")

  const getActionBadge = (action: FilterRule["action"]) => {
    switch (action) {
      case "allow": return <Badge className="bg-green-500">Allow</Badge>
      case "block": return <Badge variant="destructive">Block</Badge>
      case "rate_limit": return <Badge className="bg-yellow-500">Rate Limit</Badge>
      case "challenge": return <Badge className="bg-blue-500">Challenge</Badge>
    }
  }

  const getTypeBadge = (type: FilterRule["type"]) => <Badge variant="outline">{type.toUpperCase()}</Badge>

  const toggleFilter = (id: string) => setFilters(filters.map((f) => f.id === id ? { ...f, enabled: !f.enabled } : f))

  const filteredFilters = activeTab === "all" ? filters : filters.filter((f) => f.type === activeTab)

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div><h1 className="text-2xl font-bold tracking-tight">Filters</h1><p className="text-muted-foreground">Configure and manage DDoS protection filter rules.</p></div>
        <Dialog open={isAddDialogOpen} onOpenChange={setIsAddDialogOpen}>
          <DialogTrigger asChild><Button><Plus className="mr-2 h-4 w-4" />Add Filter</Button></DialogTrigger>
          <DialogContent className="sm:max-w-[500px]">
            <DialogHeader><DialogTitle>Add New Filter Rule</DialogTitle><DialogDescription>Create a new filter rule for DDoS protection.</DialogDescription></DialogHeader>
            <div className="grid gap-4 py-4">
              <div className="grid gap-2"><Label htmlFor="name">Rule Name</Label><Input id="name" placeholder="Block SYN Flood" /></div>
              <div className="grid grid-cols-2 gap-4">
                <div className="grid gap-2"><Label htmlFor="type">Protocol Type</Label><Select><SelectTrigger><SelectValue placeholder="Select type" /></SelectTrigger><SelectContent><SelectItem value="tcp">TCP</SelectItem><SelectItem value="udp">UDP</SelectItem><SelectItem value="http">HTTP</SelectItem><SelectItem value="quic">QUIC</SelectItem></SelectContent></Select></div>
                <div className="grid gap-2"><Label htmlFor="action">Action</Label><Select><SelectTrigger><SelectValue placeholder="Select action" /></SelectTrigger><SelectContent><SelectItem value="allow">Allow</SelectItem><SelectItem value="block">Block</SelectItem><SelectItem value="rate_limit">Rate Limit</SelectItem><SelectItem value="challenge">Challenge</SelectItem></SelectContent></Select></div>
              </div>
              <div className="grid gap-2"><Label htmlFor="priority">Priority (higher = processed first)</Label><Input id="priority" type="number" placeholder="100" /></div>
              <div className="grid gap-2"><Label htmlFor="conditions">Conditions</Label><Input id="conditions" placeholder="e.g., SYN rate > 10000/s per IP" /></div>
              <div className="flex items-center space-x-2"><Switch id="enabled" defaultChecked /><Label htmlFor="enabled">Enable immediately</Label></div>
            </div>
            <DialogFooter><Button variant="outline" onClick={() => setIsAddDialogOpen(false)}>Cancel</Button><Button onClick={() => setIsAddDialogOpen(false)}>Create Filter</Button></DialogFooter>
          </DialogContent>
        </Dialog>
      </div>

      <div className="grid gap-4 md:grid-cols-4">
        <Card><CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2"><CardTitle className="text-sm font-medium">Total Filters</CardTitle><Filter className="h-4 w-4 text-muted-foreground" /></CardHeader><CardContent><div className="text-2xl font-bold">{filters.length}</div></CardContent></Card>
        <Card><CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2"><CardTitle className="text-sm font-medium">Active</CardTitle><Shield className="h-4 w-4 text-green-500" /></CardHeader><CardContent><div className="text-2xl font-bold">{filters.filter((f) => f.enabled).length}</div></CardContent></Card>
        <Card><CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2"><CardTitle className="text-sm font-medium">Total Hits</CardTitle><Zap className="h-4 w-4 text-muted-foreground" /></CardHeader><CardContent><div className="text-2xl font-bold">{(filters.reduce((acc, f) => acc + f.hits, 0) / 1000).toFixed(0)}K</div></CardContent></Card>
        <Card><CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2"><CardTitle className="text-sm font-medium">Block Rules</CardTitle><Server className="h-4 w-4 text-red-500" /></CardHeader><CardContent><div className="text-2xl font-bold">{filters.filter((f) => f.action === "block").length}</div></CardContent></Card>
      </div>

      <Card>
        <CardHeader><CardTitle>Filter Rules</CardTitle><CardDescription>Manage your protection filter rules by protocol type.</CardDescription></CardHeader>
        <CardContent>
          <Tabs value={activeTab} onValueChange={setActiveTab}>
            <TabsList><TabsTrigger value="all"><Globe className="mr-2 h-4 w-4" />All</TabsTrigger><TabsTrigger value="tcp">TCP</TabsTrigger><TabsTrigger value="udp">UDP</TabsTrigger><TabsTrigger value="http">HTTP</TabsTrigger><TabsTrigger value="quic">QUIC</TabsTrigger></TabsList>
            <TabsContent value={activeTab} className="mt-4">
              <Table>
                <TableHeader><TableRow><TableHead className="w-[50px]">Active</TableHead><TableHead>Name</TableHead><TableHead>Type</TableHead><TableHead>Action</TableHead><TableHead>Priority</TableHead><TableHead>Conditions</TableHead><TableHead className="text-right">Hits</TableHead><TableHead className="w-[50px]"></TableHead></TableRow></TableHeader>
                <TableBody>
                  {filteredFilters.map((filter) => (
                    <TableRow key={filter.id}>
                      <TableCell><Switch checked={filter.enabled} onCheckedChange={() => toggleFilter(filter.id)} /></TableCell>
                      <TableCell className="font-medium">{filter.name}</TableCell>
                      <TableCell>{getTypeBadge(filter.type)}</TableCell>
                      <TableCell>{getActionBadge(filter.action)}</TableCell>
                      <TableCell>{filter.priority}</TableCell>
                      <TableCell className="max-w-[200px] truncate text-muted-foreground">{filter.conditions}</TableCell>
                      <TableCell className="text-right">{filter.hits.toLocaleString()}</TableCell>
                      <TableCell><DropdownMenu><DropdownMenuTrigger asChild><Button variant="ghost" size="icon"><MoreVertical className="h-4 w-4" /></Button></DropdownMenuTrigger><DropdownMenuContent align="end"><DropdownMenuItem><Pencil className="mr-2 h-4 w-4" />Edit</DropdownMenuItem><DropdownMenuItem className="text-destructive"><Trash2 className="mr-2 h-4 w-4" />Delete</DropdownMenuItem></DropdownMenuContent></DropdownMenu></TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </TabsContent>
          </Tabs>
        </CardContent>
      </Card>
    </div>
  )
}
