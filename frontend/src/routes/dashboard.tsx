import { createFileRoute, Outlet } from "@tanstack/react-router"
import { SidebarProvider, SidebarInset, SidebarTrigger } from "@/components/ui/sidebar"
import { AppSidebar } from "@/components/app-sidebar"
import { Separator } from "@/components/ui/separator"

export const Route = createFileRoute("/dashboard")({ component: DashboardLayout })

function DashboardLayout() {
  return (
    <SidebarProvider>
      <AppSidebar />
      <SidebarInset>
        <header className="flex h-14 shrink-0 items-center gap-2 border-b px-4">
          <SidebarTrigger className="-ml-1" />
          <Separator orientation="vertical" className="mr-2 h-4" />
          <span className="font-medium">PistonProtection</span>
        </header>
        <main className="flex-1 overflow-auto p-4"><Outlet /></main>
      </SidebarInset>
    </SidebarProvider>
  )
}
