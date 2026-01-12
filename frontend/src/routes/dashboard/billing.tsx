import { createFileRoute } from "@tanstack/react-router"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  CardFooter,
} from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Progress } from "@/components/ui/progress"
import { Separator } from "@/components/ui/separator"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import {
  CreditCard,
  Download,
  Check,
  Zap,
  Shield,
  Server,
  Activity,
  ArrowRight,
} from "lucide-react"

export const Route = createFileRoute("/dashboard/billing")({
  component: BillingPage,
})

const invoices = [
  { id: "INV-001", date: "Jan 1, 2025", amount: "$299.00", status: "paid" },
  { id: "INV-002", date: "Dec 1, 2024", amount: "$299.00", status: "paid" },
  { id: "INV-003", date: "Nov 1, 2024", amount: "$299.00", status: "paid" },
  { id: "INV-004", date: "Oct 1, 2024", amount: "$199.00", status: "paid" },
  { id: "INV-005", date: "Sep 1, 2024", amount: "$199.00", status: "paid" },
]

const plans = [
  {
    name: "Basic",
    price: "$99",
    description: "For small projects and testing",
    features: [
      "Up to 5 backends",
      "1M requests/month",
      "Basic DDoS protection",
      "Email support",
      "1 user",
    ],
    current: false,
  },
  {
    name: "Standard",
    price: "$199",
    description: "For growing businesses",
    features: [
      "Up to 15 backends",
      "10M requests/month",
      "Advanced DDoS protection",
      "Priority email support",
      "5 users",
      "Analytics dashboard",
    ],
    current: false,
  },
  {
    name: "Enterprise",
    price: "$299",
    description: "For large-scale operations",
    features: [
      "Unlimited backends",
      "Unlimited requests",
      "Enterprise DDoS protection",
      "24/7 phone support",
      "Unlimited users",
      "Advanced analytics",
      "Custom filters",
      "Dedicated account manager",
    ],
    current: true,
  },
]

function BillingPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold tracking-tight">Billing</h1>
        <p className="text-muted-foreground">
          Manage your subscription and billing information.
        </p>
      </div>

      {/* Current Plan */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="flex items-center gap-2">
                <Zap className="h-5 w-5 text-yellow-500" />
                Current Plan
              </CardTitle>
              <CardDescription>
                Your subscription renews on February 1, 2025
              </CardDescription>
            </div>
            <Badge className="text-lg px-4 py-1">Enterprise</Badge>
          </div>
        </CardHeader>
        <CardContent>
          <div className="grid gap-6 md:grid-cols-4">
            <div className="space-y-2">
              <p className="text-sm text-muted-foreground">Backends Used</p>
              <div className="flex items-center gap-2">
                <Server className="h-4 w-4" />
                <span className="text-2xl font-bold">12</span>
                <span className="text-muted-foreground">/ Unlimited</span>
              </div>
            </div>
            <div className="space-y-2">
              <p className="text-sm text-muted-foreground">Requests This Month</p>
              <div className="flex items-center gap-2">
                <Activity className="h-4 w-4" />
                <span className="text-2xl font-bold">45.2M</span>
                <span className="text-muted-foreground">/ Unlimited</span>
              </div>
            </div>
            <div className="space-y-2">
              <p className="text-sm text-muted-foreground">Users</p>
              <div className="flex items-center gap-2">
                <span className="text-2xl font-bold">8</span>
                <span className="text-muted-foreground">/ Unlimited</span>
              </div>
            </div>
            <div className="space-y-2">
              <p className="text-sm text-muted-foreground">Protection Level</p>
              <div className="flex items-center gap-2">
                <Shield className="h-4 w-4 text-green-500" />
                <span className="text-2xl font-bold">Enterprise</span>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Usage */}
      <Card>
        <CardHeader>
          <CardTitle>Usage This Month</CardTitle>
          <CardDescription>
            Your resource consumption for the current billing period.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          <div className="space-y-2">
            <div className="flex items-center justify-between text-sm">
              <span>Protected Traffic</span>
              <span>45.2M / Unlimited requests</span>
            </div>
            <Progress value={45} />
          </div>
          <div className="space-y-2">
            <div className="flex items-center justify-between text-sm">
              <span>Blocked Attacks</span>
              <span>1.28M attacks blocked</span>
            </div>
            <Progress value={100} className="bg-red-100" />
          </div>
          <div className="space-y-2">
            <div className="flex items-center justify-between text-sm">
              <span>API Calls</span>
              <span>234,567 / Unlimited calls</span>
            </div>
            <Progress value={23} />
          </div>
        </CardContent>
      </Card>

      {/* Available Plans */}
      <div className="space-y-4">
        <h2 className="text-xl font-semibold">Available Plans</h2>
        <div className="grid gap-4 md:grid-cols-3">
          {plans.map((plan) => (
            <Card
              key={plan.name}
              className={plan.current ? "border-primary" : ""}
            >
              <CardHeader>
                <div className="flex items-center justify-between">
                  <CardTitle>{plan.name}</CardTitle>
                  {plan.current && <Badge>Current Plan</Badge>}
                </div>
                <CardDescription>{plan.description}</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="flex items-baseline gap-1">
                  <span className="text-4xl font-bold">{plan.price}</span>
                  <span className="text-muted-foreground">/month</span>
                </div>
                <Separator />
                <ul className="space-y-2">
                  {plan.features.map((feature, i) => (
                    <li key={i} className="flex items-center gap-2 text-sm">
                      <Check className="h-4 w-4 text-green-500" />
                      {feature}
                    </li>
                  ))}
                </ul>
              </CardContent>
              <CardFooter>
                {plan.current ? (
                  <Button className="w-full" variant="outline" disabled>
                    Current Plan
                  </Button>
                ) : (
                  <Button className="w-full" variant="outline">
                    {plan.price === "$299" ? "Downgrade" : "Upgrade"}
                    <ArrowRight className="ml-2 h-4 w-4" />
                  </Button>
                )}
              </CardFooter>
            </Card>
          ))}
        </div>
      </div>

      {/* Payment Method */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <CreditCard className="h-5 w-5" />
            Payment Method
          </CardTitle>
          <CardDescription>
            Manage your payment information.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between p-4 border rounded-lg">
            <div className="flex items-center gap-4">
              <div className="w-12 h-8 bg-gradient-to-r from-blue-600 to-blue-800 rounded flex items-center justify-center text-white text-xs font-bold">
                VISA
              </div>
              <div>
                <p className="font-medium">Visa ending in 4242</p>
                <p className="text-sm text-muted-foreground">Expires 12/2026</p>
              </div>
            </div>
            <Button variant="outline" size="sm">
              Update
            </Button>
          </div>
          <Button variant="outline">
            <CreditCard className="mr-2 h-4 w-4" />
            Add Payment Method
          </Button>
        </CardContent>
      </Card>

      {/* Invoices */}
      <Card>
        <CardHeader>
          <CardTitle>Invoice History</CardTitle>
          <CardDescription>
            Download invoices for your records.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Invoice</TableHead>
                <TableHead>Date</TableHead>
                <TableHead>Amount</TableHead>
                <TableHead>Status</TableHead>
                <TableHead className="text-right">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {invoices.map((invoice) => (
                <TableRow key={invoice.id}>
                  <TableCell className="font-medium">{invoice.id}</TableCell>
                  <TableCell>{invoice.date}</TableCell>
                  <TableCell>{invoice.amount}</TableCell>
                  <TableCell>
                    <Badge
                      variant={invoice.status === "paid" ? "default" : "secondary"}
                      className={invoice.status === "paid" ? "bg-green-500" : ""}
                    >
                      {invoice.status}
                    </Badge>
                  </TableCell>
                  <TableCell className="text-right">
                    <Button variant="ghost" size="sm">
                      <Download className="h-4 w-4" />
                    </Button>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </CardContent>
      </Card>

      {/* Billing Address */}
      <Card>
        <CardHeader>
          <CardTitle>Billing Address</CardTitle>
          <CardDescription>
            This address will appear on your invoices.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-2">
          <p className="font-medium">Acme Corporation</p>
          <p className="text-muted-foreground">
            123 Main Street<br />
            Suite 400<br />
            San Francisco, CA 94102<br />
            United States
          </p>
          <Button variant="outline" className="mt-4">
            Update Address
          </Button>
        </CardContent>
      </Card>
    </div>
  )
}
