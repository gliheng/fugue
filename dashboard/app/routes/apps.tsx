import { useState } from "react";
import { Card, Input, ToggleButton, ToggleButtonGroup } from "@heroui/react";
import { Icon } from "@iconify/react";
import { useQuery } from "@tanstack/react-query";
import { Link } from "react-router";
import { api } from "../lib/api";
import { AppCard } from "../components/app-card";
import type { Route } from "./+types/apps";

export function meta({}: Route.MetaArgs) {
  return [{ title: "Apps - Fugue Dashboard" }];
}

export default function Apps() {
  const [filter, setFilter] = useState("");
  const [statusFilter, setStatusFilter] = useState<string | null>(null);

  const { data: apps, isLoading } = useQuery({
    queryKey: ["apps"],
    queryFn: () => api.listApps(),
  });

  const filteredApps = apps?.filter((app) => {
    const matchesName = app.name.toLowerCase().includes(filter.toLowerCase());
    const matchesStatus = !statusFilter || app.status === statusFilter;
    return matchesName && matchesStatus;
  });

  return (
    <div className="p-8 max-w-6xl mx-auto">
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-3xl font-bold">Apps</h1>
          <p className="text-muted mt-1">Manage your applications</p>
        </div>
      </div>

      <div className="flex items-center gap-3 mb-6">
        <Input
          placeholder="Search apps..."
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
          className="w-64"
          variant="secondary"
        />
        <ToggleButtonGroup
          selectionMode="single"
          disallowEmptySelection
          selectedKeys={statusFilter ? new Set([statusFilter]) : new Set(["all"])}
          onSelectionChange={(keys) => {
            const selected = Array.from(keys)[0] as string;
            setStatusFilter(selected === "all" ? null : selected);
          }}
        >
          <ToggleButton id="all">All</ToggleButton>
          <ToggleButton id="running"><ToggleButtonGroup.Separator />Running</ToggleButton>
          <ToggleButton id="stopped"><ToggleButtonGroup.Separator />Stopped</ToggleButton>
          <ToggleButton id="error"><ToggleButtonGroup.Separator />Error</ToggleButton>
          <ToggleButton id="building"><ToggleButtonGroup.Separator />Building</ToggleButton>
        </ToggleButtonGroup>
      </div>

      {isLoading ? (
        <div className="flex items-center justify-center py-20 text-muted">
          <Icon icon="lucide:loader-2" className="w-6 h-6 animate-spin mr-2" />
          Loading apps...
        </div>
      ) : filteredApps && filteredApps.length > 0 ? (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {filteredApps.map((app) => (
            <AppCard key={app.id} app={app} />
          ))}
        </div>
      ) : (
        <div className="text-center py-20">
          <Icon icon="lucide:inbox" className="w-12 h-12 text-muted mx-auto mb-4" />
          <p className="text-muted">No apps found</p>
          <Link to="/workspace" className="text-accent mt-2 inline-block hover:underline">
            Create your first app in the workspace
          </Link>
        </div>
      )}
    </div>
  );
}
