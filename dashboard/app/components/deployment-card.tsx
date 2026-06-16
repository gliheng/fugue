import { Card, Chip } from "@heroui/react";
import { Icon } from "@iconify/react";
import { Link } from "react-router";
import { StatusBadge } from "./status-badge";
import type { App } from "../lib/types";

const frameworkIcons: Record<string, string> = {
  "react-router": "lucide:route",
  nuxtjs: "lucide:nuxt",
  vite: "lucide:zap",
  "worker": "lucide:file-code",
  hono: "lucide:flame",
};

export function DeploymentCard({ app }: { app: App }) {
  return (
    <Link to={`/deployments/${app.id}`} className="block no-underline">
      <Card className="w-full hover:shadow-lg transition-shadow cursor-pointer">
        <Card.Header className="flex items-start justify-between">
          <div className="flex items-center gap-3">
            <div className="flex items-center justify-center w-10 h-10 rounded-xl bg-accent-soft text-accent">
              <Icon
                icon={frameworkIcons[app.framework] ?? "lucide:code-2"}
                className="w-5 h-5"
              />
            </div>
            <div>
              <Card.Title className="text-base">{app.name}</Card.Title>
              <Card.Description className="text-xs">{app.framework}</Card.Description>
            </div>
          </div>
          <StatusBadge status={app.status} />
        </Card.Header>
        <Card.Content>
          <div className="flex items-center gap-4 text-sm text-muted">
            <span className="flex items-center gap-1">
              <Icon icon="lucide:globe" className="w-3 h-3" />
              {app.subdomain}.{window.location.hostname.replace(/^dashboard\./, "")}
            </span>
            <span className="flex items-center gap-1">
              <Icon icon="lucide:clock" className="w-3 h-3" />
              {new Date(app.created_at).toLocaleDateString()}
            </span>
          </div>
          {app.description && (
            <p className="mt-2 text-sm text-muted line-clamp-2">{app.description}</p>
          )}
        </Card.Content>
      </Card>
    </Link>
  );
}
