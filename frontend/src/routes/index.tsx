import { createFileRoute, Link } from "@tanstack/react-router"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Shield, Zap, Globe, Server, ArrowRight, Check } from "lucide-react"

export const Route = createFileRoute("/")({ component: LandingPage })

function LandingPage() {
  const features = [
    { icon: Shield, title: "Advanced DDoS Protection", description: "eBPF/XDP-powered packet filtering stops attacks at the network edge before they reach your infrastructure." },
    { icon: Zap, title: "Sub-millisecond Latency", description: "Our kernel-level protection adds virtually no latency to your legitimate traffic." },
    { icon: Globe, title: "Global Anycast Network", description: "Traffic is absorbed and filtered across our worldwide network of scrubbing centers." },
    { icon: Server, title: "Multi-Protocol Support", description: "Protect TCP, UDP, HTTP, and QUIC traffic with protocol-aware filtering rules." },
  ]
  const plans = [
    { name: "Starter", price: 49, features: ["1 TB Bandwidth", "10M Requests/month", "5 Backends", "Email Support", "Basic Filters"], cta: "Get Started" },
    { name: "Professional", price: 199, features: ["5 TB Bandwidth", "100M Requests/month", "15 Backends", "Priority Support", "Custom Filters", "Analytics Dashboard"], popular: true, cta: "Start Free Trial" },
    { name: "Enterprise", price: 499, features: ["10 TB Bandwidth", "200M Requests/month", "25 Backends", "24/7 Support", "Custom Filters", "Dedicated IP", "99.99% SLA"], cta: "Contact Sales" },
  ]
  return (
    <div className="min-h-screen bg-background">
      <header className="border-b">
        <div className="container mx-auto flex h-16 items-center justify-between px-4">
          <div className="flex items-center gap-2"><Shield className="h-6 w-6 text-primary" /><span className="font-bold text-xl">PistonProtection</span></div>
          <nav className="hidden md:flex items-center gap-6"><a href="#features" className="text-sm text-muted-foreground hover:text-foreground">Features</a><a href="#pricing" className="text-sm text-muted-foreground hover:text-foreground">Pricing</a><a href="#" className="text-sm text-muted-foreground hover:text-foreground">Docs</a></nav>
          <div className="flex items-center gap-4"><Link to="/auth/login"><Button variant="ghost">Sign In</Button></Link><Link to="/auth/register"><Button>Get Started</Button></Link></div>
        </div>
      </header>
      <main>
        <section className="py-20 md:py-32">
          <div className="container mx-auto px-4 text-center">
            <Badge className="mb-4" variant="secondary">Now with QUIC Protection</Badge>
            <h1 className="text-4xl md:text-6xl font-bold tracking-tight mb-6">Enterprise DDoS Protection<br /><span className="text-primary">Powered by eBPF</span></h1>
            <p className="text-xl text-muted-foreground max-w-2xl mx-auto mb-8">Stop volumetric attacks before they overwhelm your infrastructure. Our kernel-level XDP filtering provides unmatched protection with minimal latency impact.</p>
            <div className="flex flex-col sm:flex-row gap-4 justify-center"><Link to="/auth/register"><Button size="lg">Start Free Trial<ArrowRight className="ml-2 h-4 w-4" /></Button></Link><Link to="/dashboard"><Button size="lg" variant="outline">View Dashboard</Button></Link></div>
            <div className="flex items-center justify-center gap-8 mt-12 text-sm text-muted-foreground">
              <div className="flex items-center gap-2"><Check className="h-4 w-4 text-green-500" />No credit card required</div>
              <div className="flex items-center gap-2"><Check className="h-4 w-4 text-green-500" />14-day free trial</div>
              <div className="flex items-center gap-2"><Check className="h-4 w-4 text-green-500" />Cancel anytime</div>
            </div>
          </div>
        </section>
        <section id="features" className="py-20 bg-muted/50">
          <div className="container mx-auto px-4">
            <div className="text-center mb-12"><h2 className="text-3xl font-bold mb-4">Built for Modern Infrastructure</h2><p className="text-muted-foreground max-w-2xl mx-auto">Our eBPF/XDP technology operates at the kernel level, providing protection that traditional solutions simply cannot match.</p></div>
            <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-4">
              {features.map((f, i) => (<Card key={i}><CardHeader><f.icon className="h-10 w-10 text-primary mb-2" /><CardTitle>{f.title}</CardTitle></CardHeader><CardContent><CardDescription>{f.description}</CardDescription></CardContent></Card>))}
            </div>
          </div>
        </section>
        <section className="py-20">
          <div className="container mx-auto px-4">
            <div className="grid md:grid-cols-2 gap-12 items-center">
              <div>
                <h2 className="text-3xl font-bold mb-4">Real-Time Attack Visibility</h2>
                <p className="text-muted-foreground mb-6">Monitor your traffic in real-time with our comprehensive analytics dashboard. See exactly what's being blocked and why.</p>
                <ul className="space-y-3">{["Live traffic visualization", "Geographic attack mapping", "Protocol-level analysis", "Custom alerting rules"].map((item, i) => (<li key={i} className="flex items-center gap-2"><Check className="h-5 w-5 text-green-500" />{item}</li>))}</ul>
              </div>
              <Card className="p-6"><div className="space-y-4"><div className="flex items-center justify-between"><span className="text-sm font-medium">Blocked Attacks (24h)</span><Badge variant="destructive">+28%</Badge></div><div className="text-4xl font-bold">16,382</div><div className="h-2 bg-muted rounded"><div className="h-2 bg-destructive rounded w-3/4" /></div><div className="grid grid-cols-2 gap-4 pt-4"><div><div className="text-sm text-muted-foreground">SYN Floods</div><div className="text-xl font-bold">5,234</div></div><div><div className="text-sm text-muted-foreground">UDP Amplification</div><div className="text-xl font-bold">3,456</div></div></div></div></Card>
            </div>
          </div>
        </section>
        <section id="pricing" className="py-20 bg-muted/50">
          <div className="container mx-auto px-4">
            <div className="text-center mb-12"><h2 className="text-3xl font-bold mb-4">Simple, Transparent Pricing</h2><p className="text-muted-foreground">Choose the plan that fits your infrastructure needs.</p></div>
            <div className="grid gap-6 md:grid-cols-3 max-w-5xl mx-auto">
              {plans.map((p, i) => (
                <Card key={i} className={p.popular ? "border-primary shadow-lg" : ""}>
                  <CardHeader>{p.popular && <Badge className="w-fit mb-2">Most Popular</Badge>}<CardTitle>{p.name}</CardTitle><CardDescription><span className="text-4xl font-bold">${p.price}</span><span className="text-muted-foreground">/month</span></CardDescription></CardHeader>
                  <CardContent className="space-y-4"><ul className="space-y-2">{p.features.map((f, j) => (<li key={j} className="flex items-center gap-2 text-sm"><Check className="h-4 w-4 text-green-500" />{f}</li>))}</ul><Button className="w-full" variant={p.popular ? "default" : "outline"}>{p.cta}</Button></CardContent>
                </Card>
              ))}
            </div>
          </div>
        </section>
        <section className="py-20">
          <div className="container mx-auto px-4 text-center">
            <h2 className="text-3xl font-bold mb-4">Ready to protect your infrastructure?</h2>
            <p className="text-muted-foreground mb-8 max-w-xl mx-auto">Join thousands of companies that trust PistonProtection to keep their services online.</p>
            <Link to="/auth/register"><Button size="lg">Start Your Free Trial<ArrowRight className="ml-2 h-4 w-4" /></Button></Link>
          </div>
        </section>
      </main>
      <footer className="border-t py-12">
        <div className="container mx-auto px-4">
          <div className="flex flex-col md:flex-row items-center justify-between gap-4">
            <div className="flex items-center gap-2"><Shield className="h-5 w-5 text-primary" /><span className="font-bold">PistonProtection</span></div>
            <div className="flex items-center gap-6 text-sm text-muted-foreground"><a href="#" className="hover:text-foreground">Privacy</a><a href="#" className="hover:text-foreground">Terms</a><a href="#" className="hover:text-foreground">Documentation</a><a href="#" className="hover:text-foreground">Support</a></div>
            <p className="text-sm text-muted-foreground">© 2025 PistonProtection. All rights reserved.</p>
          </div>
        </div>
      </footer>
    </div>
  )
}
