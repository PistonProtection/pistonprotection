import { createFileRoute } from "@tanstack/react-router"
import { useState } from "react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "@/components/ui/dropdown-menu"
import { Server, Plus, MoreVertical, Pencil, Trash2, Activity, Globe, Shield } from "lucide-react"

export const Route = createFileRoute("/dashboard/backends")({ component: BackendsPage })

const mockBackends = [
  { id: "1", name: "Main API Server", address: "api.example.com", port: 443, protocol: "https", status: "healthy", protectionLevel: "enterprise", requestsPerSec: 15420, blockedThreats: 234 },
  { id: "2", name: "Game Server US-East", address: "192.168.1.100", port: 25565, protocol: "tcp", status: "healthy", protectionLevel: "standard", requestsPerSec: 8930, blockedThreats: 89 },
  { id: "3", name: "Voice Server", address: "voice.example.com", port: 9987, protocol: "udp", status: "degraded", protectionLevel: "basic", requestsPerSec: 2100, blockedThreats: 12 },
  { id: "4", name: "Web Frontend", address: "www.example.com", port: 443, protocol: "https", status: "healthy", protectionLevel: "enterprise", requestsPerSec: 32100, blockedThreats: 567 },
  { id: "5", name: "Game Server EU-West", address: "192.168.2.50", port: 25565, protocol: "tcp", status: "unhealthy", protectionLevel: "standard", requestsPerSec: 0, blockedThreats: 0 },
]

function BackendsPage() {
  const [backends] = useState(mockBackends)
  const [isAddDialogOpen, setIsAddDialogOpen] = useState(false)
  const getStatusBadge = (s: string) => s==="healthy"?<Badge className="bg-green-500">Healthy</Badge>:s==="degraded"?<Badge className="bg-yellow-500">Degraded</Badge>:<Badge variant="destructive">Unhealthy</Badge>
  const getProtectionBadge = (l: string) => l==="basic"?<Badge variant="secondary">Basic</Badge>:l==="standard"?<Badge variant="outline">Standard</Badge>:<Badge>Enterprise</Badge>
  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div><h1 className="text-2xl font-bold tracking-tight">Backends</h1><p className="text-muted-foreground">Manage your protected backend servers.</p></div>
        <Dialog open={isAddDialogOpen} onOpenChange={setIsAddDialogOpen}>
          <DialogTrigger asChild><Button><Plus className="mr-2 h-4 w-4" />Add Backend</Button></DialogTrigger>
          <DialogContent className="sm:max-w-[500px]">
            <DialogHeader><DialogTitle>Add New Backend</DialogTitle><DialogDescription>Configure a new backend server for DDoS protection.</DialogDescription></DialogHeader>
            <div className="grid gap-4 py-4">
              <div className="grid gap-2"><Label htmlFor="name">Name</Label><Input id="name" placeholder="My Server" /></div>
              <div className="grid grid-cols-2 gap-4"><div className="grid gap-2"><Label>Address</Label><Input placeholder="192.168.1.1" /></div><div className="grid gap-2"><Label>Port</Label><Input type="number" placeholder="443" /></div></div>
              <div className="grid grid-cols-2 gap-4"><div className="grid gap-2"><Label>Protocol</Label><Select><SelectTrigger><SelectValue placeholder="Select" /></SelectTrigger><SelectContent><SelectItem value="tcp">TCP</SelectItem><SelectItem value="udp">UDP</SelectItem><SelectItem value="http">HTTP</SelectItem><SelectItem value="https">HTTPS</SelectItem></SelectContent></Select></div><div className="grid gap-2"><Label>Protection</Label><Select><SelectTrigger><SelectValue placeholder="Select" /></SelectTrigger><SelectContent><SelectItem value="basic">Basic</SelectItem><SelectItem value="standard">Standard</SelectItem><SelectItem value="enterprise">Enterprise</SelectItem></SelectContent></Select></div></div>
            </div>
            <DialogFooter><Button variant="outline" onClick={()=>setIsAddDialogOpen(false)}>Cancel</Button><Button onClick={()=>setIsAddDialogOpen(false)}>Add Backend</Button></DialogFooter>
          </DialogContent>
        </Dialog>
      </div>
      <div className="grid gap-4 md:grid-cols-3">
        <Card><CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2"><CardTitle className="text-sm font-medium">Total Backends</CardTitle><Server className="h-4 w-4 text-muted-foreground" /></CardHeader><CardContent><div className="text-2xl font-bold">{backends.length}</div></CardContent></Card>
        <Card><CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2"><CardTitle className="text-sm font-medium">Healthy</CardTitle><Activity className="h-4 w-4 text-green-500" /></CardHeader><CardContent><div className="text-2xl font-bold">{backends.filter(b=>b.status==="healthy").length}</div></CardContent></Card>
        <Card><CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2"><CardTitle className="text-sm font-medium">Protected Traffic</CardTitle><Shield className="h-4 w-4 text-muted-foreground" /></CardHeader><CardContent><div className="text-2xl font-bold">{(backends.reduce((a,b)=>a+b.requestsPerSec,0)/1000).toFixed(1)}K/s</div></CardContent></Card>
      </div>
      <Card><CardHeader><CardTitle>All Backends</CardTitle><CardDescription>View and manage all your protected backend servers.</CardDescription></CardHeader><CardContent>
        <Table><TableHeader><TableRow><TableHead>Name</TableHead><TableHead>Address</TableHead><TableHead>Protocol</TableHead><TableHead>Status</TableHead><TableHead>Protection</TableHead><TableHead className="text-right">Req/s</TableHead><TableHead className="text-right">Blocked</TableHead><TableHead className="w-[50px]"></TableHead></TableRow></TableHeader>
          <TableBody>{backends.map(b=>(<TableRow key={b.id}><TableCell className="font-medium"><div className="flex items-center gap-2"><Globe className="h-4 w-4 text-muted-foreground" />{b.name}</div></TableCell><TableCell>{b.address}:{b.port}</TableCell><TableCell className="uppercase">{b.protocol}</TableCell><TableCell>{getStatusBadge(b.status)}</TableCell><TableCell>{getProtectionBadge(b.protectionLevel)}</TableCell><TableCell className="text-right">{b.requestsPerSec.toLocaleString()}</TableCell><TableCell className="text-right">{b.blockedThreats.toLocaleString()}</TableCell><TableCell><DropdownMenu><DropdownMenuTrigger asChild><Button variant="ghost" size="icon"><MoreVertical className="h-4 w-4" /></Button></DropdownMenuTrigger><DropdownMenuContent align="end"><DropdownMenuItem><Pencil className="mr-2 h-4 w-4" />Edit</DropdownMenuItem><DropdownMenuItem className="text-destructive"><Trash2 className="mr-2 h-4 w-4" />Delete</DropdownMenuItem></DropdownMenuContent></DropdownMenu></TableCell></TableRow>))}</TableBody>
        </Table>
      </CardContent></Card>
    </div>
  )
}
