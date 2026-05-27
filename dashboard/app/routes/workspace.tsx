import { useState } from "react";
import { Button, Modal, Spinner, Input } from "@heroui/react";
import { Icon } from "@iconify/react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useNavigate, Link } from "react-router";
import { api } from "../lib/api";
import type { Route } from "./+types/workspace";

export function meta({}: Route.MetaArgs) {
  return [{ title: "Workspace - Fugue Dashboard" }];
}

const FRAMEWORK_TEMPLATES = [
  { id: "react-router", name: "React Router", icon: "lucide:route", desc: "React Router v7 with SSR" },
  { id: "nuxtjs", name: "Nuxt.js", icon: "lucide:hexagon", desc: "Full-stack Nuxt.js with SSR" },
  { id: "worker", name: "Worker", icon: "lucide:file-code", desc: "Simple Cloudflare Worker" },
];

const FRAMEWORK_ICONS: Record<string, string> = {
  "react-router": "lucide:route",
  nuxtjs: "lucide:hexagon",
  worker: "lucide:file-code",
};

export default function WorkspaceList() {
  const [templateOpen, setTemplateOpen] = useState(false);
  const [creating, setCreating] = useState(false);
  const [deleteId, setDeleteId] = useState<string | null>(null);
  const [renameId, setRenameId] = useState<string | null>(null);
  const [renameName, setRenameName] = useState("");

  const navigate = useNavigate();
  const queryClient = useQueryClient();

  const { data: workspaces, isLoading } = useQuery({
    queryKey: ["workspaces"],
    queryFn: () => api.listWorkspaces(),
  });

  const handleCreateFromTemplate = async (framework: string) => {
    setCreating(true);
    try {
      const ws = await api.createWorkspace({ framework });
      await queryClient.invalidateQueries({ queryKey: ["workspaces"] });
      setTemplateOpen(false);
      navigate(`/workspace/${ws.id}`);
    } catch (e) {
      console.error("Failed to create workspace:", e);
    } finally {
      setCreating(false);
    }
  };

  const handleDelete = async () => {
    if (!deleteId) return;
    try {
      await api.deleteWorkspace(deleteId);
      await queryClient.invalidateQueries({ queryKey: ["workspaces"] });
      setDeleteId(null);
    } catch (e) {
      console.error("Failed to delete workspace:", e);
    }
  };

  const handleRename = async () => {
    if (!renameId || !renameName.trim()) return;
    try {
      await api.updateWorkspace(renameId, { name: renameName.trim() });
      await queryClient.invalidateQueries({ queryKey: ["workspaces"] });
      setRenameId(null);
    } catch (e) {
      console.error("Failed to rename workspace:", e);
    }
  };

  return (
    <div className="p-8 max-w-6xl mx-auto">
      <div className="mb-8">
        <h1 className="text-3xl font-bold">Workspace</h1>
        <p className="text-muted mt-1">Create and deploy applications from templates</p>
      </div>

      <div className="flex items-center justify-between mb-6">
        <h2 className="text-lg font-semibold">Your Workspaces</h2>
        <Button onPress={() => setTemplateOpen(true)}>
          <Icon icon="lucide:plus" className="w-4 h-4" />
          Create App
        </Button>
      </div>

      {isLoading ? (
        <div className="flex items-center justify-center py-20">
          <Spinner size="lg" />
        </div>
      ) : workspaces && workspaces.length > 0 ? (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {workspaces.map((ws) => (
            <div key={ws.id} className="relative">
              <Link
                to={`/workspace/${ws.id}`}
                className="block rounded-xl border border-border p-4 hover:border-accent hover:bg-accent-soft/30 transition-colors no-underline"
              >
                <div className="flex items-center gap-3 mb-2">
                  <div className="flex items-center justify-center w-10 h-10 rounded-xl bg-accent-soft">
                    <Icon icon={FRAMEWORK_ICONS[ws.framework] ?? "lucide:code-2"} className="w-5 h-5 text-accent" />
                  </div>
                  <div className="min-w-0 flex-1">
                    <p className="text-sm font-semibold truncate text-foreground">{ws.name}</p>
                    <p className="text-xs text-muted">{ws.framework}</p>
                  </div>
                </div>
                <p className="text-xs text-muted">
                  {ws.file_count} files · Updated {new Date(ws.updated_at).toLocaleDateString()}
                </p>
              </Link>
              <Button
                size="sm"
                variant="ghost"
                className="absolute top-2 right-8"
                onPress={() => { setRenameId(ws.id); setRenameName(ws.name); }}
              >
                <Icon icon="lucide:pencil" className="w-3 h-3" />
              </Button>
              <Button
                size="sm"
                variant="ghost"
                className="absolute top-2 right-2"
                onPress={() => setDeleteId(ws.id)}
              >
                <Icon icon="lucide:trash-2" className="w-3 h-3" />
              </Button>
            </div>
          ))}
        </div>
      ) : (
        <div className="flex flex-col items-center justify-center py-20">
          <div className="flex items-center justify-center w-16 h-16 rounded-2xl bg-accent-soft mb-6">
            <Icon icon="lucide:plus-circle" className="w-8 h-8 text-accent" />
          </div>
          <h3 className="text-lg font-semibold mb-2">No workspaces yet</h3>
          <p className="text-muted text-sm mb-6 text-center max-w-md">
            Pick a template, customize the code, and deploy it to an app
          </p>
          <Button color="accent" size="lg" onPress={() => setTemplateOpen(true)}>
            <Icon icon="lucide:plus" className="w-5 h-5" />
            Create App
          </Button>
        </div>
      )}

      <Modal.Backdrop isOpen={templateOpen} onOpenChange={setTemplateOpen}>
        <Modal.Container>
          <Modal.Dialog className="sm:max-w-lg">
            <Modal.CloseTrigger />
            <Modal.Header>
              <Modal.Heading>Choose a Template</Modal.Heading>
              <p className="text-sm text-muted">Select a framework to start from</p>
            </Modal.Header>
            <Modal.Body>
              <div className="grid grid-cols-1 gap-3">
                {creating && (
                  <div className="flex items-center justify-center py-4">
                    <Spinner size="lg" />
                  </div>
                )}
                {FRAMEWORK_TEMPLATES.map((t) => (
                  <button
                    key={t.id}
                    type="button"
                    className="w-full text-left p-4 rounded-xl border border-border hover:border-accent hover:bg-accent-soft transition-colors flex items-center gap-3 disabled:opacity-50"
                    onClick={() => handleCreateFromTemplate(t.id)}
                    disabled={creating}
                  >
                    <div className="flex items-center justify-center w-10 h-10 rounded-xl bg-accent-soft">
                      <Icon icon={t.icon} className="w-5 h-5 text-accent" />
                    </div>
                    <div>
                      <p className="text-sm font-semibold">{t.name}</p>
                      <p className="text-xs text-muted">{t.desc}</p>
                    </div>
                  </button>
                ))}
              </div>
            </Modal.Body>
            <Modal.Footer>
              <Button slot="close" variant="secondary">Cancel</Button>
            </Modal.Footer>
          </Modal.Dialog>
        </Modal.Container>
      </Modal.Backdrop>

      <Modal.Backdrop isOpen={deleteId !== null} onOpenChange={(open) => { if (!open) setDeleteId(null); }}>
        <Modal.Container>
          <Modal.Dialog className="sm:max-w-[360px]">
            <Modal.Header>
              <Modal.Icon className="bg-danger-soft text-danger-soft-foreground">
                <Icon icon="lucide:trash-2" className="size-5" />
              </Modal.Icon>
              <Modal.Heading>Delete workspace?</Modal.Heading>
            </Modal.Header>
            <Modal.Body>
              <p className="text-sm text-muted">
                This will permanently delete the workspace and all its files. This action cannot be undone.
              </p>
            </Modal.Body>
            <Modal.Footer>
              <Button slot="close" variant="secondary">Cancel</Button>
              <Button variant="danger" onPress={handleDelete}>Delete</Button>
            </Modal.Footer>
          </Modal.Dialog>
        </Modal.Container>
      </Modal.Backdrop>

      <Modal.Backdrop isOpen={renameId !== null} onOpenChange={(open) => { if (!open) setRenameId(null); }}>
        <Modal.Container>
          <Modal.Dialog className="sm:max-w-[400px]">
            <Modal.CloseTrigger />
            <Modal.Header>
              <Modal.Heading>Rename workspace</Modal.Heading>
            </Modal.Header>
            <Modal.Body>
              <Input
                label="Name"
                value={renameName}
                onChange={(e) => setRenameName(e.target.value)}
                onKeyDown={(e) => { if (e.key === "Enter") handleRename(); }}
                autoFocus
              />
            </Modal.Body>
            <Modal.Footer>
              <Button slot="close" variant="secondary">Cancel</Button>
              <Button onPress={handleRename} isDisabled={!renameName.trim()}>Rename</Button>
            </Modal.Footer>
          </Modal.Dialog>
        </Modal.Container>
      </Modal.Backdrop>
    </div>
  );
}