import { createFileRoute, Link, useNavigate } from "@tanstack/react-router"
import { useState } from "react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle, CardFooter } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Checkbox } from "@/components/ui/checkbox"
import { Separator } from "@/components/ui/separator"
import { Shield, Github, Mail, Check } from "lucide-react"

export const Route = createFileRoute("/auth/register")({ component: RegisterPage })

function RegisterPage() {
  const navigate = useNavigate()
  const [isLoading, setIsLoading] = useState(false)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setIsLoading(true)
    setTimeout(() => { setIsLoading(false); navigate({ to: "/dashboard" }) }, 1000)
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-background p-4">
      <div className="w-full max-w-md space-y-6">
        <div className="flex flex-col items-center space-y-2 text-center">
          <div className="flex items-center gap-2">
            <div className="flex aspect-square size-10 items-center justify-center rounded-lg bg-primary text-primary-foreground"><Shield className="size-6" /></div>
            <span className="text-2xl font-bold">PistonProtection</span>
          </div>
          <p className="text-muted-foreground">Enterprise DDoS Protection Platform</p>
        </div>

        <Card>
          <CardHeader className="space-y-1"><CardTitle className="text-2xl">Create an account</CardTitle><CardDescription>Start protecting your infrastructure today</CardDescription></CardHeader>
          <CardContent>
            <form onSubmit={handleSubmit} className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2"><Label htmlFor="firstName">First name</Label><Input id="firstName" placeholder="John" required /></div>
                <div className="space-y-2"><Label htmlFor="lastName">Last name</Label><Input id="lastName" placeholder="Doe" required /></div>
              </div>
              <div className="space-y-2"><Label htmlFor="email">Email</Label><Input id="email" type="email" placeholder="name@example.com" required /></div>
              <div className="space-y-2"><Label htmlFor="company">Company name</Label><Input id="company" placeholder="Acme Inc." /></div>
              <div className="space-y-2">
                <Label htmlFor="password">Password</Label>
                <Input id="password" type="password" required />
                <ul className="text-xs text-muted-foreground space-y-1 mt-2">
                  <li className="flex items-center gap-1"><Check className="h-3 w-3 text-green-500" />At least 12 characters</li>
                  <li className="flex items-center gap-1"><Check className="h-3 w-3 text-green-500" />One uppercase letter</li>
                  <li className="flex items-center gap-1"><Check className="h-3 w-3 text-green-500" />One number</li>
                </ul>
              </div>
              <div className="space-y-2"><Label htmlFor="confirmPassword">Confirm password</Label><Input id="confirmPassword" type="password" required /></div>
              <div className="flex items-start space-x-2"><Checkbox id="terms" required /><Label htmlFor="terms" className="text-sm font-normal leading-tight">I agree to the <Link to="/" className="text-primary hover:underline">Terms of Service</Link> and <Link to="/" className="text-primary hover:underline">Privacy Policy</Link></Label></div>
              <Button type="submit" className="w-full" disabled={isLoading}>{isLoading ? "Creating account..." : "Create account"}</Button>
            </form>

            <div className="relative my-4">
              <div className="absolute inset-0 flex items-center"><Separator className="w-full" /></div>
              <div className="relative flex justify-center text-xs uppercase"><span className="bg-card px-2 text-muted-foreground">Or continue with</span></div>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <Button variant="outline" type="button"><Github className="mr-2 h-4 w-4" />GitHub</Button>
              <Button variant="outline" type="button"><Mail className="mr-2 h-4 w-4" />Google</Button>
            </div>
          </CardContent>
          <CardFooter><p className="text-sm text-muted-foreground text-center w-full">Already have an account? <Link to="/auth/login" className="text-primary hover:underline">Sign in</Link></p></CardFooter>
        </Card>

        <div className="text-center space-y-2">
          <p className="text-sm font-medium">Included in all plans:</p>
          <ul className="text-xs text-muted-foreground space-y-1">
            <li className="flex items-center justify-center gap-1"><Check className="h-3 w-3 text-green-500" />14-day free trial</li>
            <li className="flex items-center justify-center gap-1"><Check className="h-3 w-3 text-green-500" />No credit card required</li>
            <li className="flex items-center justify-center gap-1"><Check className="h-3 w-3 text-green-500" />Cancel anytime</li>
          </ul>
        </div>
      </div>
    </div>
  )
}
