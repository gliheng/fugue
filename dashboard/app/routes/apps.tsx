import { useState } from "react";
import { Button, Card, Input, Modal, ToggleButton, ToggleButtonGroup } from "@heroui/react";
import { Icon } from "@iconify/react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Link, useNavigate } from "react-router";
import { api } from "../lib/api";
import { AppCard } from "../components/app-card";
import type { Route } from "./+types/apps";
import type { CreateAppRequest } from "../lib/types";

export function meta({}: Route.MetaArgs) {
  return [{ title: "Apps - Fugue Dashboard" }];
}

export default function Apps() {
  const [createOpen, setCreateOpen] = useState(false);
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
        <Button onPress={() => setCreateOpen(true)}>
          <Icon icon="lucide:plus" className="w-4 h-4" />
          Create App
        </Button>
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
          <Button className="mt-4" onPress={() => setCreateOpen(true)}>
            Create your first app
          </Button>
        </div>
      )}

      <CreateAppModal open={createOpen} onClose={() => setCreateOpen(false)} />
    </div>
  );
}

function CreateAppModal({
  open,
  onClose,
}: {
  open: boolean;
  onClose: () => void;
}) {
  const [name, setName] = useState("");
  const [framework, setFramework] = useState("react-router");
  const [description, setDescription] = useState("");
  const queryClient = useQueryClient();
  const navigate = useNavigate();

  const createMutation = useMutation({
    mutationFn: (data: CreateAppRequest) => api.createApp(data),
    onSuccess: (app) => {
      queryClient.invalidateQueries({ queryKey: ["apps"] });
      setName("");
      setFramework("react-router");
      setDescription("");
      onClose();
      navigate(`/apps/${app.id}?tab=code`);
    },
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    createMutation.mutate({ name, framework, description: description || undefined });
  };

  return (
    <Modal.Backdrop isOpen={open} onOpenChange={(isOpen) => { if (!isOpen) onClose(); }}>
      <Modal.Container>
        <Modal.Dialog className="sm:max-w-md">
          <Modal.CloseTrigger />
          <Modal.Header>
            <Modal.Heading>Create App</Modal.Heading>
            <p className="text-sm text-muted">Set up a new application on Fugue</p>
          </Modal.Header>
          <Modal.Body>
            <form onSubmit={handleSubmit} className="flex flex-col gap-4">
              <div>
                <label className="text-sm font-medium mb-1 block">Name</label>
                <Input
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  placeholder="my-app"
                  variant="secondary"
                  required
                />
              </div>
              <div>
                <label className="text-sm font-medium mb-1 block">Framework</label>
                <ToggleButtonGroup
                  selectionMode="single"
                  disallowEmptySelection
                  selectedKeys={new Set([framework])}
                  onSelectionChange={(keys) => setFramework(Array.from(keys)[0] as string)}
                >
                  <ToggleButton id="react-router">React Router</ToggleButton>
                  <ToggleButton id="nuxtjs"><ToggleButtonGroup.Separator />Nuxt.js</ToggleButton>
                  <ToggleButton id="worker"><ToggleButtonGroup.Separator />Worker</ToggleButton>
                </ToggleButtonGroup>
              </div>
              <div>
                <label className="text-sm font-medium mb-1 block">Description (optional)</label>
                <Input
                  value={description}
                  onChange={(e) => setDescription(e.target.value)}
                  placeholder="Describe your app..."
                  variant="secondary"
                />
              </div>
              {createMutation.error && (
                <p className="text-sm text-danger">
                  {createMutation.error instanceof Error ? createMutation.error.message : "Failed to create app"}
                </p>
              )}
              <div className="flex justify-end gap-3 mt-2">
                <Button variant="ghost" onPress={onClose}>
                  Cancel
                </Button>
                <Button type="submit" isDisabled={!name.trim() || createMutation.isPending}>
                  {createMutation.isPending ? "Creating..." : "Create"}
                </Button>
              </div>
            </form>
          </Modal.Body>
        </Modal.Dialog>
      </Modal.Container>
    </Modal.Backdrop>
  );
}
