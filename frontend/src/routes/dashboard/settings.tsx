import { createFileRoute } from "@tanstack/react-router"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Switch } from "@/components/ui/switch"
import { Separator } from "@/components/ui/separator"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Textarea } from "@/components/ui/textarea"
import {
  Bell,
  Shield,
  Key,
  Globe,
  Mail,
  Webhook,
  AlertTriangle,
} from "lucide-react"

export const Route = createFileRoute("/dashboard/settings")({
  component: SettingsPage,
})

function SettingsPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold tracking-tight">Settings</h1>
        <p className="text-muted-foreground">
          Configure your DDoS protection settings and preferences.
        </p>
      </div>

      <Tabs defaultValue="general" className="space-y-4">
        <TabsList>
          <TabsTrigger value="general">General</TabsTrigger>
          <TabsTrigger value="security">Security</TabsTrigger>
          <TabsTrigger value="notifications">Notifications</TabsTrigger>
          <TabsTrigger value="api">API</TabsTrigger>
        </TabsList>

        {/* General Settings */}
        <TabsContent value="general" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Organization</CardTitle>
              <CardDescription>
                Manage your organization settings.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid gap-2">
                <Label htmlFor="org-name">Organization Name</Label>
                <Input id="org-name" defaultValue="Acme Corp" />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="org-email">Contact Email</Label>
                <Input
                  id="org-email"
                  type="email"
                  defaultValue="admin@acme.com"
                />
              </div>
              <Button>Save Changes</Button>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Shield className="h-5 w-5" />
                Protection Defaults
              </CardTitle>
              <CardDescription>
                Default settings for new backends.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid gap-2">
                <Label htmlFor="default-level">Default Protection Level</Label>
                <Select defaultValue="standard">
                  <SelectTrigger>
                    <SelectValue placeholder="Select level" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="basic">Basic</SelectItem>
                    <SelectItem value="standard">Standard</SelectItem>
                    <SelectItem value="enterprise">Enterprise</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="flex items-center justify-between">
                <div className="space-y-0.5">
                  <Label>Auto-enable TCP Protection</Label>
                  <p className="text-sm text-muted-foreground">
                    Automatically enable TCP protection for new backends.
                  </p>
                </div>
                <Switch defaultChecked />
              </div>
              <div className="flex items-center justify-between">
                <div className="space-y-0.5">
                  <Label>Auto-enable UDP Protection</Label>
                  <p className="text-sm text-muted-foreground">
                    Automatically enable UDP protection for new backends.
                  </p>
                </div>
                <Switch defaultChecked />
              </div>
              <div className="flex items-center justify-between">
                <div className="space-y-0.5">
                  <Label>Auto-enable HTTP Protection</Label>
                  <p className="text-sm text-muted-foreground">
                    Automatically enable HTTP/HTTPS protection for new backends.
                  </p>
                </div>
                <Switch defaultChecked />
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Globe className="h-5 w-5" />
                IP Allowlist
              </CardTitle>
              <CardDescription>
                IPs in this list will bypass all protection filters.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid gap-2">
                <Label htmlFor="allowlist">Allowed IPs (one per line)</Label>
                <Textarea
                  id="allowlist"
                  placeholder="192.168.1.1&#10;10.0.0.0/8&#10;172.16.0.0/12"
                  className="min-h-[100px] font-mono"
                  defaultValue="10.0.0.0/8&#10;172.16.0.0/12&#10;192.168.0.0/16"
                />
              </div>
              <Button>Update Allowlist</Button>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Security Settings */}
        <TabsContent value="security" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Key className="h-5 w-5" />
                Two-Factor Authentication
              </CardTitle>
              <CardDescription>
                Add an extra layer of security to your account.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex items-center justify-between">
                <div className="space-y-0.5">
                  <Label>Enable 2FA</Label>
                  <p className="text-sm text-muted-foreground">
                    Require two-factor authentication for all users.
                  </p>
                </div>
                <Switch />
              </div>
              <Separator />
              <div className="flex items-center justify-between">
                <div className="space-y-0.5">
                  <Label>Require 2FA for API Access</Label>
                  <p className="text-sm text-muted-foreground">
                    Require 2FA verification for API key generation.
                  </p>
                </div>
                <Switch />
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <AlertTriangle className="h-5 w-5" />
                Security Policies
              </CardTitle>
              <CardDescription>
                Configure security policies for your organization.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid gap-2">
                <Label htmlFor="session-timeout">Session Timeout (minutes)</Label>
                <Input
                  id="session-timeout"
                  type="number"
                  defaultValue="60"
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="password-expiry">Password Expiry (days)</Label>
                <Input
                  id="password-expiry"
                  type="number"
                  defaultValue="90"
                />
              </div>
              <div className="flex items-center justify-between">
                <div className="space-y-0.5">
                  <Label>Enforce Strong Passwords</Label>
                  <p className="text-sm text-muted-foreground">
                    Require minimum 12 characters with special characters.
                  </p>
                </div>
                <Switch defaultChecked />
              </div>
              <Button>Save Security Settings</Button>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Notification Settings */}
        <TabsContent value="notifications" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Mail className="h-5 w-5" />
                Email Notifications
              </CardTitle>
              <CardDescription>
                Configure email alerts for security events.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex items-center justify-between">
                <div className="space-y-0.5">
                  <Label>Attack Alerts</Label>
                  <p className="text-sm text-muted-foreground">
                    Receive email when an attack is detected.
                  </p>
                </div>
                <Switch defaultChecked />
              </div>
              <div className="flex items-center justify-between">
                <div className="space-y-0.5">
                  <Label>Backend Health Alerts</Label>
                  <p className="text-sm text-muted-foreground">
                    Receive email when a backend goes unhealthy.
                  </p>
                </div>
                <Switch defaultChecked />
              </div>
              <div className="flex items-center justify-between">
                <div className="space-y-0.5">
                  <Label>Weekly Reports</Label>
                  <p className="text-sm text-muted-foreground">
                    Receive weekly traffic and security reports.
                  </p>
                </div>
                <Switch defaultChecked />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="alert-email">Alert Recipients</Label>
                <Input
                  id="alert-email"
                  type="email"
                  placeholder="alerts@example.com"
                  defaultValue="security@acme.com"
                />
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Webhook className="h-5 w-5" />
                Webhook Notifications
              </CardTitle>
              <CardDescription>
                Send events to external services via webhooks.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex items-center justify-between">
                <div className="space-y-0.5">
                  <Label>Enable Webhooks</Label>
                  <p className="text-sm text-muted-foreground">
                    Send attack events to your webhook endpoint.
                  </p>
                </div>
                <Switch />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="webhook-url">Webhook URL</Label>
                <Input
                  id="webhook-url"
                  placeholder="https://your-service.com/webhook"
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="webhook-secret">Webhook Secret</Label>
                <Input
                  id="webhook-secret"
                  type="password"
                  placeholder="Enter secret key"
                />
              </div>
              <Button>Save Webhook Settings</Button>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Bell className="h-5 w-5" />
                Alert Thresholds
              </CardTitle>
              <CardDescription>
                Configure when to trigger alerts.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid gap-2">
                <Label htmlFor="attack-threshold">
                  Attack Alert Threshold (blocked requests/min)
                </Label>
                <Input
                  id="attack-threshold"
                  type="number"
                  defaultValue="1000"
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="traffic-spike">
                  Traffic Spike Threshold (% increase)
                </Label>
                <Input
                  id="traffic-spike"
                  type="number"
                  defaultValue="200"
                />
              </div>
              <Button>Save Thresholds</Button>
            </CardContent>
          </Card>
        </TabsContent>

        {/* API Settings */}
        <TabsContent value="api" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Key className="h-5 w-5" />
                API Keys
              </CardTitle>
              <CardDescription>
                Manage your API keys for programmatic access.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="rounded-lg border p-4 space-y-2">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="font-medium">Production Key</p>
                    <p className="text-sm text-muted-foreground">
                      Created: Jan 1, 2025
                    </p>
                  </div>
                  <Button variant="outline" size="sm">
                    Revoke
                  </Button>
                </div>
                <code className="block p-2 bg-muted rounded text-sm font-mono">
                  pp_live_****************************abcd
                </code>
              </div>
              <div className="rounded-lg border p-4 space-y-2">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="font-medium">Development Key</p>
                    <p className="text-sm text-muted-foreground">
                      Created: Dec 15, 2024
                    </p>
                  </div>
                  <Button variant="outline" size="sm">
                    Revoke
                  </Button>
                </div>
                <code className="block p-2 bg-muted rounded text-sm font-mono">
                  pp_test_****************************wxyz
                </code>
              </div>
              <Button>Generate New API Key</Button>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>API Access Control</CardTitle>
              <CardDescription>
                Configure allowed origins and rate limits.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid gap-2">
                <Label htmlFor="allowed-origins">Allowed Origins</Label>
                <Textarea
                  id="allowed-origins"
                  placeholder="https://example.com&#10;https://app.example.com"
                  className="min-h-[80px] font-mono"
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="rate-limit">Rate Limit (requests/minute)</Label>
                <Input
                  id="rate-limit"
                  type="number"
                  defaultValue="1000"
                />
              </div>
              <Button>Save API Settings</Button>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  )
}
