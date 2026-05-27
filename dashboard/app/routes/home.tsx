import { Card, Chip } from "@heroui/react";
import { Icon } from "@iconify/react";
import { Link } from "react-router";
import { useQuery } from "@tanstack/react-query";
import { api } from "../lib/api";
import type { Route } from "./+types/home";

export function meta({}: Route.MetaArgs) {
  return [
    { title: "Fugue Dashboard" },
    { name: "description", content: "Manage your Fugue applications" },
  ];
}

export default function Home() {
  const { data: apps } = useQuery({
    queryKey: ["apps"],
    queryFn: () => api.listApps(),
  });

  const { data: platformStatus } = useQuery({
    queryKey: ["platform-status"],
    queryFn: () => api.platformStatus(),
  });

  const runningApps = apps?.filter((a) => a.status === "running").length ?? 0;
  const totalApps = apps?.length ?? 0;

  return (
    <div className="p-8 max-w-6xl mx-auto">
      <div className="mb-8">
        <h1 className="text-3xl font-bold">Dashboard</h1>
        <p className="text-muted mt-1">Welcome to the Fugue platform</p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
        <Card>
          <Card.Header className="flex items-center gap-3">
            <div className="flex items-center justify-center w-10 h-10 rounded-xl bg-accent-soft">
              <Icon icon="lucide:boxes" className="w-5 h-5 text-accent" />
            </div>
            <div>
              <Card.Title>Total Apps</Card.Title>
              <Card.Description>All applications</Card.Description>
            </div>
          </Card.Header>
          <Card.Content>
            <p className="text-3xl font-bold">{totalApps}</p>
          </Card.Content>
        </Card>

        <Card>
          <Card.Header className="flex items-center gap-3">
            <div className="flex items-center justify-center w-10 h-10 rounded-xl bg-success-soft">
              <Icon icon="lucide:play-circle" className="w-5 h-5 text-success" />
            </div>
            <div>
              <Card.Title>Running</Card.Title>
              <Card.Description>Active applications</Card.Description>
            </div>
          </Card.Header>
          <Card.Content>
            <p className="text-3xl font-bold">{runningApps}</p>
          </Card.Content>
        </Card>

        <Card>
          <Card.Header className="flex items-center gap-3">
            <div className="flex items-center justify-center w-10 h-10 rounded-xl bg-warning-soft">
              <Icon icon="lucide:server" className="w-5 h-5 text-warning" />
            </div>
            <div>
              <Card.Title>Platform</Card.Title>
              <Card.Description>Status</Card.Description>
            </div>
          </Card.Header>
          <Card.Content>
            <Chip color="success" size="sm" variant="soft">
              {platformStatus?.status ?? "—"}
            </Chip>
          </Card.Content>
        </Card>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <Card>
          <Card.Header>
            <Card.Title>Quick Actions</Card.Title>
          </Card.Header>
          <Card.Content className="flex flex-col gap-3">
            <Link
              to="/workspace"
              className="flex items-center gap-3 p-3 rounded-lg bg-surface-tertiary hover:bg-accent-soft transition-colors"
            >
              <Icon icon="lucide:code-2" className="w-5 h-5 text-accent" />
              <div>
                <p className="font-medium text-sm">Workspace</p>
                <p className="text-xs text-muted">Create and deploy from templates</p>
              </div>
            </Link>
            <Link
              to="/deployments"
              className="flex items-center gap-3 p-3 rounded-lg bg-surface-tertiary hover:bg-accent-soft transition-colors"
            >
              <Icon icon="lucide:boxes" className="w-5 h-5 text-accent" />
              <div>
                <p className="font-medium text-sm">Manage Deployments</p>
                <p className="text-xs text-muted">View and manage deployments</p>
              </div>
            </Link>
          </Card.Content>
        </Card>

        <Card>
          <Card.Header>
            <Card.Title>Recent Apps</Card.Title>
          </Card.Header>
          <Card.Content>
            {apps && apps.length > 0 ? (
              <div className="space-y-3">
                {apps.slice(0, 5).map((app) => (
                  <Link
                    key={app.id}
                    to={`/deployments/${app.id}`}
                    className="flex items-center justify-between p-2 rounded-lg hover:bg-surface-tertiary transition-colors"
                  >
                    <div className="flex items-center gap-3">
                      <Icon
                        icon={
                          app.framework === "react-router"
                            ? "lucide:route"
                            : app.framework === "nuxtjs"
                              ? "lucide:hexagon"
                              : "lucide:file-code"
                        }
                        className="w-4 h-4 text-muted"
                      />
                      <span className="text-sm font-medium">{app.name}</span>
                    </div>
                    <Chip
                      color={
                        app.status === "running"
                          ? "success"
                          : app.status === "error"
                            ? "danger"
                            : "default"
                      }
                      size="sm"
                      variant="soft"
                    >
                      {app.status}
                    </Chip>
                  </Link>
                ))}
              </div>
            ) : (
              <p className="text-sm text-muted text-center py-8">No apps yet. Create your first app!</p>
            )}
          </Card.Content>
        </Card>
      </div>
    </div>
  );
}
