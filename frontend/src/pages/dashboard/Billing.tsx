import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import {
  CreditCard,
  Check,
  Zap,
  Shield,
  Server,
  Download,
  ExternalLink,
  Loader2,
  AlertTriangle,
} from 'lucide-react'
import { toast } from 'sonner'

import { Button } from '@/components/ui/button'
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Badge } from '@/components/ui/badge'
import { Progress } from '@/components/ui/progress'
import { Separator } from '@/components/ui/separator'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Skeleton } from '@/components/ui/skeleton'
import { subscriptionQueryOptions, apiClient } from '@/lib/api'
import { cn } from '@/lib/utils'

const plans = [
  {
    id: 'free',
    name: 'Free',
    price: 0,
    description: 'For personal projects and testing',
    features: [
      '1 backend server',
      '100K requests/month',
      'Basic DDoS protection',
      '24-hour data retention',
      'Community support',
    ],
    limits: {
      backends: 1,
      requests: 100000,
      retention: 1,
    },
  },
  {
    id: 'starter',
    name: 'Starter',
    price: 29,
    description: 'For small teams and applications',
    features: [
      '3 backend servers',
      '1M requests/month',
      'Advanced DDoS protection',
      '7-day data retention',
      'Email support',
      'API access',
    ],
    limits: {
      backends: 3,
      requests: 1000000,
      retention: 7,
    },
  },
  {
    id: 'pro',
    name: 'Pro',
    price: 99,
    description: 'For growing businesses',
    features: [
      '10 backend servers',
      '10M requests/month',
      'Enterprise DDoS protection',
      '30-day data retention',
      'Priority support',
      'Custom filter rules',
      'Advanced analytics',
    ],
    limits: {
      backends: 10,
      requests: 10000000,
      retention: 30,
    },
    popular: true,
  },
  {
    id: 'enterprise',
    name: 'Enterprise',
    price: -1,
    description: 'For large organizations',
    features: [
      'Unlimited backends',
      'Unlimited requests',
      'Custom protection solutions',
      '1-year data retention',
      'Dedicated support',
      'SLA guarantee',
      'Custom integrations',
      'On-premise deployment',
    ],
    limits: {
      backends: Infinity,
      requests: Infinity,
      retention: 365,
    },
  },
]

function PlanCard({
  plan,
  currentPlan,
  onSelect,
  isLoading,
}: {
  plan: (typeof plans)[0]
  currentPlan: string
  onSelect: (planId: string) => void
  isLoading: boolean
}) {
  const isCurrent = currentPlan === plan.id
  const isEnterprise = plan.price === -1

  return (
    <Card
      className={cn(
        'relative flex flex-col',
        plan.popular && 'border-primary shadow-lg',
        isCurrent && 'ring-2 ring-primary'
      )}
    >
      {plan.popular && (
        <Badge className="absolute -top-3 left-1/2 -translate-x-1/2">
          Most Popular
        </Badge>
      )}
      <CardHeader>
        <CardTitle className="flex items-center justify-between">
          {plan.name}
          {isCurrent && (
            <Badge variant="secondary">Current Plan</Badge>
          )}
        </CardTitle>
        <CardDescription>{plan.description}</CardDescription>
      </CardHeader>
      <CardContent className="flex-1">
        <div className="mb-4">
          {isEnterprise ? (
            <div className="text-3xl font-bold">Custom</div>
          ) : (
            <div className="flex items-baseline gap-1">
              <span className="text-4xl font-bold">${plan.price}</span>
              <span className="text-muted-foreground">/month</span>
            </div>
          )}
        </div>
        <ul className="space-y-2">
          {plan.features.map((feature, i) => (
            <li key={i} className="flex items-center gap-2 text-sm">
              <Check className="h-4 w-4 text-success" />
              {feature}
            </li>
          ))}
        </ul>
      </CardContent>
      <CardFooter>
        <Button
          className="w-full"
          variant={plan.popular ? 'default' : 'outline'}
          disabled={isCurrent || isLoading}
          onClick={() => onSelect(plan.id)}
        >
          {isLoading ? (
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
          ) : null}
          {isCurrent
            ? 'Current Plan'
            : isEnterprise
            ? 'Contact Sales'
            : plan.price === 0
            ? 'Downgrade'
            : 'Upgrade'}
        </Button>
      </CardFooter>
    </Card>
  )
}

function CurrentPlanCard() {
  const { data: subscription, isLoading } = useQuery(subscriptionQueryOptions())

  const portalMutation = useMutation({
    mutationFn: () => apiClient.createPortalSession(),
    onSuccess: (data) => {
      window.open(data.url, '_blank')
    },
    onError: (error: Error) => {
      toast.error(`Failed to open billing portal: ${error.message}`)
    },
  })

  if (isLoading) {
    return (
      <Card>
        <CardHeader>
          <Skeleton className="h-6 w-32" />
          <Skeleton className="h-4 w-48" />
        </CardHeader>
        <CardContent>
          <Skeleton className="h-20 w-full" />
        </CardContent>
      </Card>
    )
  }

  const plan = subscription?.plan
  const usage = subscription?.usage

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle className="flex items-center gap-2">
              <Zap className="h-5 w-5 text-primary" />
              {plan?.name || 'Free'} Plan
            </CardTitle>
            <CardDescription>
              {subscription?.status === 'active'
                ? `Next billing date: ${new Date(subscription.nextBillingDate).toLocaleDateString()}`
                : subscription?.status === 'canceling'
                ? 'Cancels at end of billing period'
                : 'No active subscription'}
            </CardDescription>
          </div>
          <Badge
            variant={
              subscription?.status === 'active'
                ? 'success'
                : subscription?.status === 'canceling'
                ? 'warning'
                : 'secondary'
            }
          >
            {subscription?.status || 'Free'}
          </Badge>
        </div>
      </CardHeader>
      <CardContent className="space-y-6">
        {/* Usage Stats */}
        <div className="grid gap-4 md:grid-cols-3">
          <div className="space-y-2">
            <div className="flex items-center justify-between text-sm">
              <span className="flex items-center gap-2">
                <Server className="h-4 w-4 text-muted-foreground" />
                Backends
              </span>
              <span>
                {usage?.backends || 0} / {plan?.backends || 1}
              </span>
            </div>
            <Progress
              value={((usage?.backends || 0) / parseInt(plan?.backends || '1')) * 100}
            />
          </div>
          <div className="space-y-2">
            <div className="flex items-center justify-between text-sm">
              <span className="flex items-center gap-2">
                <Zap className="h-4 w-4 text-muted-foreground" />
                Requests
              </span>
              <span>{usage?.requests || '0'}</span>
            </div>
            <Progress value={50} />
          </div>
          <div className="space-y-2">
            <div className="flex items-center justify-between text-sm">
              <span className="flex items-center gap-2">
                <Shield className="h-4 w-4 text-muted-foreground" />
                Protection
              </span>
              <span>{plan?.protection || '10 Gbps'}</span>
            </div>
            <Progress value={30} />
          </div>
        </div>

        {/* Payment Method */}
        {subscription?.paymentMethod && (
          <>
            <Separator />
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <div className="flex h-10 w-16 items-center justify-center rounded-md bg-muted">
                  <CreditCard className="h-6 w-6" />
                </div>
                <div>
                  <p className="font-medium">
                    {subscription.paymentMethod.brand} ending in{' '}
                    {subscription.paymentMethod.last4}
                  </p>
                  <p className="text-sm text-muted-foreground">
                    Expires {subscription.paymentMethod.expiry}
                  </p>
                </div>
              </div>
              <Button
                variant="outline"
                onClick={() => portalMutation.mutate()}
                disabled={portalMutation.isPending}
              >
                {portalMutation.isPending ? (
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                ) : (
                  <ExternalLink className="mr-2 h-4 w-4" />
                )}
                Manage
              </Button>
            </div>
          </>
        )}
      </CardContent>
    </Card>
  )
}

function InvoicesTable() {
  const { data: subscription, isLoading } = useQuery(subscriptionQueryOptions())
  const invoices = subscription?.invoices || []

  if (isLoading) {
    return (
      <Card>
        <CardHeader>
          <Skeleton className="h-6 w-32" />
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {[...Array(3)].map((_, i) => (
              <Skeleton key={i} className="h-12 w-full" />
            ))}
          </div>
        </CardContent>
      </Card>
    )
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>Billing History</CardTitle>
        <CardDescription>View and download past invoices</CardDescription>
      </CardHeader>
      <CardContent>
        {invoices.length > 0 ? (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Date</TableHead>
                <TableHead>Description</TableHead>
                <TableHead>Amount</TableHead>
                <TableHead>Status</TableHead>
                <TableHead className="w-[80px]"></TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {invoices.map((invoice) => (
                <TableRow key={invoice.id}>
                  <TableCell>
                    {new Date(invoice.date).toLocaleDateString()}
                  </TableCell>
                  <TableCell>{invoice.description}</TableCell>
                  <TableCell>
                    ${(invoice.amount / 100).toFixed(2)}{' '}
                    {invoice.currency.toUpperCase()}
                  </TableCell>
                  <TableCell>
                    <Badge
                      variant={
                        invoice.status === 'paid'
                          ? 'success'
                          : invoice.status === 'pending'
                          ? 'warning'
                          : 'secondary'
                      }
                    >
                      {invoice.status}
                    </Badge>
                  </TableCell>
                  <TableCell>
                    <Button variant="ghost" size="icon" className="h-8 w-8">
                      <Download className="h-4 w-4" />
                    </Button>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        ) : (
          <div className="flex flex-col items-center justify-center py-8 text-center">
            <CreditCard className="h-10 w-10 text-muted-foreground" />
            <p className="mt-2 text-sm text-muted-foreground">No invoices yet</p>
          </div>
        )}
      </CardContent>
    </Card>
  )
}

function CancelSubscriptionDialog({
  open,
  onOpenChange,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
}) {
  const queryClient = useQueryClient()

  const cancelMutation = useMutation({
    mutationFn: () => apiClient.cancelSubscription(),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['subscription'] })
      toast.success('Subscription will be canceled at end of billing period')
      onOpenChange(false)
    },
    onError: (error: Error) => {
      toast.error(`Failed to cancel subscription: ${error.message}`)
    },
  })

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2 text-destructive">
            <AlertTriangle className="h-5 w-5" />
            Cancel Subscription
          </DialogTitle>
          <DialogDescription>
            Are you sure you want to cancel your subscription? You will lose
            access to premium features at the end of your current billing period.
          </DialogDescription>
        </DialogHeader>
        <div className="rounded-lg bg-muted p-4">
          <p className="text-sm font-medium">What you will lose:</p>
          <ul className="mt-2 space-y-1 text-sm text-muted-foreground">
            <li>- Advanced DDoS protection</li>
            <li>- Additional backend slots</li>
            <li>- Extended data retention</li>
            <li>- Priority support</li>
          </ul>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Keep Subscription
          </Button>
          <Button
            variant="destructive"
            onClick={() => cancelMutation.mutate()}
            disabled={cancelMutation.isPending}
          >
            {cancelMutation.isPending ? (
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            ) : null}
            Cancel Subscription
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

export default function DashboardBilling() {
  const { data: subscription } = useQuery(subscriptionQueryOptions())
  const [cancelOpen, setCancelOpen] = useState(false)
  const [upgradeLoading, setUpgradeLoading] = useState<string | null>(null)

  const checkoutMutation = useMutation({
    mutationFn: (planId: string) => apiClient.createCheckoutSession(planId),
    onSuccess: (data) => {
      window.location.href = data.url
    },
    onError: (error: Error) => {
      toast.error(`Failed to start checkout: ${error.message}`)
      setUpgradeLoading(null)
    },
  })

  const handlePlanSelect = (planId: string) => {
    if (planId === 'enterprise') {
      window.open('mailto:sales@pistonprotection.com', '_blank')
      return
    }
    setUpgradeLoading(planId)
    checkoutMutation.mutate(planId)
  }

  const currentPlan = subscription?.plan?.id || 'free'

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div>
        <h1 className="text-3xl font-bold tracking-tight">Billing</h1>
        <p className="text-muted-foreground">
          Manage your subscription and billing information
        </p>
      </div>

      {/* Current Plan */}
      <CurrentPlanCard />

      {/* Plans Grid */}
      <div>
        <h2 className="mb-4 text-xl font-semibold">Available Plans</h2>
        <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-4">
          {plans.map((plan) => (
            <PlanCard
              key={plan.id}
              plan={plan}
              currentPlan={currentPlan}
              onSelect={handlePlanSelect}
              isLoading={upgradeLoading === plan.id}
            />
          ))}
        </div>
      </div>

      {/* Invoices */}
      <InvoicesTable />

      {/* Cancel Subscription */}
      {subscription?.status === 'active' && currentPlan !== 'free' && (
        <Card className="border-destructive/50">
          <CardHeader>
            <CardTitle className="text-destructive">Danger Zone</CardTitle>
            <CardDescription>
              Cancel your subscription. This action cannot be undone.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Button
              variant="destructive"
              onClick={() => setCancelOpen(true)}
            >
              Cancel Subscription
            </Button>
          </CardContent>
        </Card>
      )}

      <CancelSubscriptionDialog open={cancelOpen} onOpenChange={setCancelOpen} />
    </div>
  )
}
