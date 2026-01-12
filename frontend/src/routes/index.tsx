import { createFileRoute, Link } from "@tanstack/react-router"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import {
  Shield,
  Zap,
  Server,
  Globe,
  Lock,
  Activity,
  Check,
  ArrowRight,
} from "lucide-react"

export const Route = createFileRoute("/")({ component: LandingPage })

function LandingPage() {
  const features = [
    {
      icon: <Shield className="w-10 h-10" />,
      title: "Advanced DDoS Protection",
      description:
        "Multi-layer protection against TCP, UDP, HTTP, and QUIC attacks with eBPF/XDP filters at the kernel level.",
    },
    {
      icon: <Zap className="w-10 h-10" />,
      title: "Ultra-Low Latency",
      description:
        "XDP-based filtering processes packets before they reach the network stack, ensuring minimal latency impact.",
    },
    {
      icon: <Server className="w-10 h-10" />,
      title: "Protocol-Aware Filtering",
      description:
        "Specialized protection for gaming servers (Minecraft, QUIC), web applications, and generic TCP/UDP services.",
    },
    {
      icon: <Globe className="w-10 h-10" />,
      title: "Global Edge Network",
      description:
        "Distributed filtering across multiple edge locations to absorb attacks close to their source.",
    },
    {
      icon: <Lock className="w-10 h-10" />,
      title: "Enterprise Security",
      description:
        "SOC2 compliant with end-to-end encryption, role-based access control, and comprehensive audit logging.",
    },
    {
      icon: <Activity className="w-10 h-10" />,
      title: "Real-Time Analytics",
      description:
        "Live traffic visualization, attack detection alerts, and detailed forensic reports with Grafana integration.",
    },
  ]

  const plans = [
    {
      name: "Basic",
      price: "$99",
      description: "For small projects",
      features: [
        "5 backends",
        "1M requests/month",
        "Basic protection",
        "Email support",
      ],
    },
    {
      name: "Standard",
      price: "$199",
      description: "For growing businesses",
      features: [
        "15 backends",
        "10M requests/month",
        "Advanced protection",
        "Priority support",
        "Analytics dashboard",
      ],
      popular: true,
    },
    {
      name: "Enterprise",
      price: "$299",
      description: "For large operations",
      features: [
        "Unlimited backends",
        "Unlimited requests",
        "Enterprise protection",
        "24/7 phone support",
        "Custom filters",
        "Dedicated manager",
      ],
    },
  ]

  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <header className="border-b">
        <div className="container mx-auto px-4 h-16 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <div className="flex aspect-square size-8 items-center justify-center rounded-lg bg-primary text-primary-foreground">
              <Shield className="size-5" />
            </div>
            <span className="text-xl font-bold">PistonProtection</span>
          </div>
          <nav className="hidden md:flex items-center gap-6">
            <a href="#features" className="text-sm text-muted-foreground hover:text-foreground">
              Features
            </a>
            <a href="#pricing" className="text-sm text-muted-foreground hover:text-foreground">
              Pricing
            </a>
            <a href="#" className="text-sm text-muted-foreground hover:text-foreground">
              Documentation
            </a>
          </nav>
          <div className="flex items-center gap-2">
            <Button variant="ghost" asChild>
              <Link to="/auth/login">Sign in</Link>
            </Button>
            <Button asChild>
              <Link to="/auth/register">Get Started</Link>
            </Button>
          </div>
        </div>
      </header>

      {/* Hero */}
      <section className="py-20 px-4">
        <div className="container mx-auto text-center max-w-4xl">
          <div className="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-primary/10 text-primary text-sm mb-6">
            <Shield className="w-4 h-4" />
            Enterprise-grade DDoS Protection
          </div>
          <h1 className="text-5xl md:text-6xl font-bold tracking-tight mb-6">
            Protect Your Infrastructure from{" "}
            <span className="text-primary">DDoS Attacks</span>
          </h1>
          <p className="text-xl text-muted-foreground mb-8 max-w-2xl mx-auto">
            Advanced eBPF/XDP-based filtering technology that stops attacks at the
            kernel level. Protect your servers, APIs, and gaming infrastructure
            with ultra-low latency.
          </p>
          <div className="flex flex-col sm:flex-row items-center justify-center gap-4">
            <Button size="lg" asChild>
              <Link to="/auth/register">
                Start Free Trial
                <ArrowRight className="ml-2 h-4 w-4" />
              </Link>
            </Button>
            <Button size="lg" variant="outline" asChild>
              <Link to="/dashboard">View Demo</Link>
            </Button>
          </div>
          <p className="text-sm text-muted-foreground mt-4">
            No credit card required. 14-day free trial.
          </p>
        </div>
      </section>

      {/* Stats */}
      <section className="py-12 border-y bg-muted/30">
        <div className="container mx-auto px-4">
          <div className="grid grid-cols-2 md:grid-cols-4 gap-8 text-center">
            <div>
              <div className="text-4xl font-bold">99.99%</div>
              <div className="text-sm text-muted-foreground">Uptime SLA</div>
            </div>
            <div>
              <div className="text-4xl font-bold">&lt;1ms</div>
              <div className="text-sm text-muted-foreground">Added Latency</div>
            </div>
            <div>
              <div className="text-4xl font-bold">10Tbps+</div>
              <div className="text-sm text-muted-foreground">Mitigation Capacity</div>
            </div>
            <div>
              <div className="text-4xl font-bold">500+</div>
              <div className="text-sm text-muted-foreground">Enterprise Clients</div>
            </div>
          </div>
        </div>
      </section>

      {/* Features */}
      <section id="features" className="py-20 px-4">
        <div className="container mx-auto">
          <div className="text-center mb-12">
            <h2 className="text-3xl font-bold mb-4">Enterprise-Grade Protection</h2>
            <p className="text-muted-foreground max-w-2xl mx-auto">
              Our advanced protection stack combines multiple filtering layers to
              stop attacks before they impact your services.
            </p>
          </div>
          <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
            {features.map((feature, i) => (
              <Card key={i} className="hover:shadow-lg transition-shadow">
                <CardHeader>
                  <div className="text-primary mb-2">{feature.icon}</div>
                  <CardTitle>{feature.title}</CardTitle>
                </CardHeader>
                <CardContent>
                  <CardDescription>{feature.description}</CardDescription>
                </CardContent>
              </Card>
            ))}
          </div>
        </div>
      </section>

      {/* Pricing */}
      <section id="pricing" className="py-20 px-4 bg-muted/30">
        <div className="container mx-auto">
          <div className="text-center mb-12">
            <h2 className="text-3xl font-bold mb-4">Simple, Transparent Pricing</h2>
            <p className="text-muted-foreground">
              Choose the plan that fits your needs. All plans include a 14-day free
              trial.
            </p>
          </div>
          <div className="grid md:grid-cols-3 gap-6 max-w-5xl mx-auto">
            {plans.map((plan, i) => (
              <Card
                key={i}
                className={plan.popular ? "border-primary shadow-lg" : ""}
              >
                {plan.popular && (
                  <div className="bg-primary text-primary-foreground text-center py-1 text-sm font-medium">
                    Most Popular
                  </div>
                )}
                <CardHeader>
                  <CardTitle>{plan.name}</CardTitle>
                  <CardDescription>{plan.description}</CardDescription>
                  <div className="pt-4">
                    <span className="text-4xl font-bold">{plan.price}</span>
                    <span className="text-muted-foreground">/month</span>
                  </div>
                </CardHeader>
                <CardContent className="space-y-4">
                  <ul className="space-y-2">
                    {plan.features.map((feature, j) => (
                      <li key={j} className="flex items-center gap-2 text-sm">
                        <Check className="h-4 w-4 text-green-500" />
                        {feature}
                      </li>
                    ))}
                  </ul>
                  <Button
                    className="w-full"
                    variant={plan.popular ? "default" : "outline"}
                    asChild
                  >
                    <Link to="/auth/register">Get Started</Link>
                  </Button>
                </CardContent>
              </Card>
            ))}
          </div>
        </div>
      </section>

      {/* CTA */}
      <section className="py-20 px-4">
        <div className="container mx-auto text-center max-w-3xl">
          <h2 className="text-3xl font-bold mb-4">
            Ready to Protect Your Infrastructure?
          </h2>
          <p className="text-muted-foreground mb-8">
            Join hundreds of companies that trust PistonProtection to keep their
            services online and secure.
          </p>
          <Button size="lg" asChild>
            <Link to="/auth/register">
              Start Your Free Trial
              <ArrowRight className="ml-2 h-4 w-4" />
            </Link>
          </Button>
        </div>
      </section>

      {/* Footer */}
      <footer className="border-t py-12 px-4">
        <div className="container mx-auto">
          <div className="flex flex-col md:flex-row items-center justify-between gap-4">
            <div className="flex items-center gap-2">
              <div className="flex aspect-square size-8 items-center justify-center rounded-lg bg-primary text-primary-foreground">
                <Shield className="size-5" />
              </div>
              <span className="font-bold">PistonProtection</span>
            </div>
            <p className="text-sm text-muted-foreground">
              &copy; 2025 PistonProtection. All rights reserved.
            </p>
            <div className="flex items-center gap-4">
              <a href="#" className="text-sm text-muted-foreground hover:text-foreground">
                Terms
              </a>
              <a href="#" className="text-sm text-muted-foreground hover:text-foreground">
                Privacy
              </a>
              <a href="#" className="text-sm text-muted-foreground hover:text-foreground">
                Contact
              </a>
            </div>
          </div>
        </div>
      </footer>
    </div>
  )
}
