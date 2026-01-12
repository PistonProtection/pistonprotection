import { createFileRoute } from "@tanstack/react-router"
import { useQuery } from "@tanstack/react-query"
import {
  Check,
  CreditCard,
  Download,
  ExternalLink,
  Sparkles,
  Zap,
} from "lucide-react"
import { format } from "date-fns"

import { cn } from "@/lib/utils"
import { billingOptions, type BillingInfo } from "@/lib/api"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  CardFooter,
} from "@/components/ui/card"
import { Progress, ProgressLabel, ProgressValue } from "@/components/ui/progress"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { Separator } from "@/components/ui/separator"
import { Skeleton } from "@/components/ui/skeleton"

export const Route = createFileRoute("/dashboard/billing")({
  component: BillingPage,
})

function formatBytes(bytes: number): string {
  if (bytes >= 1024 * 1024 * 1024) {
    return (bytes / (1024 * 1024 * 1024)).toFixed(0) + " GB"
  }
  if (bytes >= 1024 * 1024) {
    return (bytes / (1024 * 1024)).toFixed(0) + " MB"
  }
  return (bytes / 1024).toFixed(0) + " KB"
}

function formatNumber(num: number): string {
  if (num >= 1_000_000) {
    return (num / 1_000_000).toFixed(1) + "M"
  }
  if (num >= 1_000) {
    return (num / 1_000).toFixed(0) + "K"
  }
  return num.toString()
}

const plans = [
  {
    name: "Free",
    price: 0,
    description: "For small projects and testing",
    features: [
      "1 Million requests/month",
      "10 GB bandwidth/month",
      "2 Backends",
      "5 Filter rules",
      "Community support",
      "Basic analytics",
    ],
    highlighted: false,
  },
  {
    name: "Pro",
    price: 99,
    description: "For growing businesses",
    features: [
      "10 Million requests/month",
      "500 GB bandwidth/month",
      "10 Backends",
      "50 Filter rules",
      "Priority support",
      "Advanced analytics",
      "Custom rules",
      "API access",
    ],
    highlighted: true,
  },
  {
    name: "Enterprise",
    price: 499,
    description: "For large-scale operations",
    features: [
      "Unlimited requests",
      "Unlimited bandwidth",
      "Unlimited backends",
      "Unlimited filter rules",
      "24/7 dedicated support",
      "Real-time analytics",
      "Custom integrations",
      "SLA guarantee",
      "On-premise option",
    ],
    highlighted: false,
  },
]

function BillingPage() {
  const { data: billing, isLoading } = useQuery(billingOptions)

  return (
    <div className="flex-1 space-y-6 p-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Billing</h1>
          <p className="text-muted-foreground">
            Manage your subscription and view usage.
          </p>
        </div>
      </div>

      {/* Current Plan */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle>Current Plan</CardTitle>
              <CardDescription>
                Your current subscription and billing cycle
              </CardDescription>
            </div>
            {isLoading ? (
              <Skeleton className="h-6 w-24" />
            ) : (
              <Badge
                variant={billing?.status === "active" ? "default" : "destructive"}
                className={
                  billing?.status === "active" ? "bg-green-600" : undefined
                }
              >
                {billing?.status === "active"
                  ? "Active"
                  : billing?.status === "past_due"
                    ? "Past Due"
                    : "Canceled"}
              </Badge>
            )}
          </div>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="space-y-4">
              <Skeleton className="h-10 w-32" />
              <Skeleton className="h-4 w-64" />
            </div>
          ) : (
            <div className="flex items-baseline gap-2">
              <span className="text-4xl font-bold">
                {billing?.plan === "free"
                  ? "Free"
                  : billing?.plan === "pro"
                    ? "$99"
                    : "$499"}
              </span>
              {billing?.plan !== "free" && (
                <span className="text-muted-foreground">/month</span>
              )}
              <Badge variant="secondary" className="ml-2">
                {billing?.plan?.toUpperCase()}
              </Badge>
            </div>
          )}
          {!isLoading && billing && (
            <p className="mt-2 text-sm text-muted-foreground">
              Current billing period:{" "}
              {format(new Date(billing.currentPeriodStart), "MMM d, yyyy")} -{" "}
              {format(new Date(billing.currentPeriodEnd), "MMM d, yyyy")}
            </p>
          )}
        </CardContent>
        <CardFooter className="border-t">
          <Button variant="outline" size="sm">
            <CreditCard className="mr-2 size-4" />
            Update Payment Method
          </Button>
        </CardFooter>
      </Card>

      {/* Usage */}
      <Card>
        <CardHeader>
          <CardTitle>Usage This Period</CardTitle>
          <CardDescription>
            Your resource consumption in the current billing cycle
          </CardDescription>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="grid gap-6 md:grid-cols-2">
              {[1, 2, 3, 4].map((i) => (
                <Skeleton key={i} className="h-20 w-full" />
              ))}
            </div>
          ) : (
            <div className="grid gap-6 md:grid-cols-2">
              <div className="space-y-2">
                <div className="flex items-center justify-between text-sm">
                  <span className="font-medium">Requests</span>
                  <span className="text-muted-foreground">
                    {formatNumber(billing?.usage.requests || 0)} /{" "}
                    {formatNumber(billing?.usage.requestsLimit || 0)}
                  </span>
                </div>
                <Progress
                  value={
                    billing
                      ? (billing.usage.requests / billing.usage.requestsLimit) *
                        100
                      : 0
                  }
                />
              </div>

              <div className="space-y-2">
                <div className="flex items-center justify-between text-sm">
                  <span className="font-medium">Bandwidth</span>
                  <span className="text-muted-foreground">
                    {formatBytes(billing?.usage.bandwidth || 0)} /{" "}
                    {formatBytes(billing?.usage.bandwidthLimit || 0)}
                  </span>
                </div>
                <Progress
                  value={
                    billing
                      ? (billing.usage.bandwidth / billing.usage.bandwidthLimit) *
                        100
                      : 0
                  }
                />
              </div>

              <div className="space-y-2">
                <div className="flex items-center justify-between text-sm">
                  <span className="font-medium">Backends</span>
                  <span className="text-muted-foreground">
                    {billing?.usage.backends || 0} /{" "}
                    {billing?.usage.backendsLimit || 0}
                  </span>
                </div>
                <Progress
                  value={
                    billing
                      ? (billing.usage.backends / billing.usage.backendsLimit) *
                        100
                      : 0
                  }
                />
              </div>

              <div className="space-y-2">
                <div className="flex items-center justify-between text-sm">
                  <span className="font-medium">Filter Rules</span>
                  <span className="text-muted-foreground">
                    {billing?.usage.filters || 0} /{" "}
                    {billing?.usage.filtersLimit || 0}
                  </span>
                </div>
                <Progress
                  value={
                    billing
                      ? (billing.usage.filters / billing.usage.filtersLimit) * 100
                      : 0
                  }
                />
              </div>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Plans */}
      <div>
        <h2 className="text-2xl font-bold tracking-tight mb-4">
          Available Plans
        </h2>
        <div className="grid gap-6 md:grid-cols-3">
          {plans.map((plan) => (
            <Card
              key={plan.name}
              className={cn(
                plan.highlighted &&
                  "border-primary ring-2 ring-primary ring-offset-2 ring-offset-background"
              )}
            >
              <CardHeader>
                <div className="flex items-center justify-between">
                  <CardTitle className="text-xl">{plan.name}</CardTitle>
                  {plan.highlighted && (
                    <Badge className="bg-primary">
                      <Sparkles className="mr-1 size-3" />
                      Popular
                    </Badge>
                  )}
                </div>
                <CardDescription>{plan.description}</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="flex items-baseline">
                  <span className="text-4xl font-bold">
                    {plan.price === 0 ? "Free" : `$${plan.price}`}
                  </span>
                  {plan.price > 0 && (
                    <span className="text-muted-foreground ml-1">/month</span>
                  )}
                </div>

                <Separator />

                <ul className="space-y-2">
                  {plan.features.map((feature, index) => (
                    <li key={index} className="flex items-center gap-2 text-sm">
                      <Check className="size-4 text-green-500 shrink-0" />
                      {feature}
                    </li>
                  ))}
                </ul>
              </CardContent>
              <CardFooter>
                <Button
                  className="w-full"
                  variant={
                    billing?.plan === plan.name.toLowerCase()
                      ? "secondary"
                      : plan.highlighted
                        ? "default"
                        : "outline"
                  }
                  disabled={billing?.plan === plan.name.toLowerCase()}
                >
                  {billing?.plan === plan.name.toLowerCase() ? (
                    "Current Plan"
                  ) : billing?.plan === "enterprise" ? (
                    "Downgrade"
                  ) : (
                    <>
                      <Zap className="mr-2 size-4" />
                      {billing?.plan === "free" ||
                      (billing?.plan === "pro" &&
                        plan.name.toLowerCase() === "enterprise")
                        ? "Upgrade"
                        : "Switch"}
                    </>
                  )}
                </Button>
              </CardFooter>
            </Card>
          ))}
        </div>
      </div>

      {/* Invoices */}
      <Card>
        <CardHeader>
          <CardTitle>Invoice History</CardTitle>
          <CardDescription>
            Download invoices for your records
          </CardDescription>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="space-y-4">
              {[1, 2, 3].map((i) => (
                <Skeleton key={i} className="h-12 w-full" />
              ))}
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Invoice</TableHead>
                  <TableHead>Date</TableHead>
                  <TableHead>Amount</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead className="w-12"></TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {billing?.invoices.map((invoice) => (
                  <TableRow key={invoice.id}>
                    <TableCell className="font-medium">{invoice.id}</TableCell>
                    <TableCell>
                      {format(new Date(invoice.date), "MMM d, yyyy")}
                    </TableCell>
                    <TableCell>${invoice.amount.toFixed(2)}</TableCell>
                    <TableCell>
                      <Badge
                        variant={
                          invoice.status === "paid"
                            ? "default"
                            : invoice.status === "pending"
                              ? "secondary"
                              : "destructive"
                        }
                        className={
                          invoice.status === "paid" ? "bg-green-600" : undefined
                        }
                      >
                        {invoice.status}
                      </Badge>
                    </TableCell>
                    <TableCell>
                      <Button variant="ghost" size="icon-sm">
                        <Download className="size-4" />
                        <span className="sr-only">Download</span>
                      </Button>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>

      {/* Help */}
      <Card>
        <CardContent className="flex items-center justify-between py-6">
          <div>
            <h3 className="font-medium">Need help with billing?</h3>
            <p className="text-sm text-muted-foreground">
              Contact our support team for billing inquiries and payment issues.
            </p>
          </div>
          <Button variant="outline">
            <ExternalLink className="mr-2 size-4" />
            Contact Support
          </Button>
        </CardContent>
      </Card>
    </div>
  )
}
