import { NavLink, Outlet } from "react-router";
import { Icon } from "@iconify/react";

const navItems = [
  { to: "/", label: "Dashboard", icon: "lucide:layout-dashboard" },
  { to: "/workspace", label: "Workspace", icon: "lucide:code-2" },
  { to: "/deployments", label: "Deployments", icon: "lucide:boxes" },
];

function Sidebar() {
  return (
    <aside className="fixed left-0 top-0 bottom-0 w-56 bg-surface-secondary border-r border-border flex flex-col">
      <div className="px-5 py-6 flex items-center gap-2">
        <svg
          className="h-7 w-7 text-accent"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
        >
          <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2" />
        </svg>
        <span className="text-lg font-bold">Fugue</span>
      </div>

      <nav className="flex-1 px-3 space-y-1">
        {navItems.map(({ to, label, icon }) => (
          <NavLink
            key={to}
            to={to}
            end={to === "/"}
            className={({ isActive }) =>
              `flex items-center gap-2 px-3 py-2 rounded-lg text-sm font-medium transition-colors ${
                isActive
                  ? "bg-accent-soft text-accent"
                  : "text-muted hover:text-foreground hover:bg-surface-tertiary"
              }`
            }
          >
            <Icon icon={icon} className="w-4 h-4" />
            {label}
          </NavLink>
        ))}
      </nav>

      <div className="px-5 py-4 border-t border-border">
        <p className="text-xs text-muted">Fugue Platform v0.1.0</p>
      </div>
    </aside>
  );
}

export default function AppLayout() {
  return (
    <div className="flex min-h-screen">
      <Sidebar />
      <main className="flex-1 ml-56">
        <Outlet />
      </main>
    </div>
  );
}
