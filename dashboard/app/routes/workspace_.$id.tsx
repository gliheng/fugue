import { useEffect, useMemo, useState } from "react";
import { useParams, Link, useNavigate, useBlocker } from "react-router";
import { Button, Input, Modal, Spinner, ToggleButton, ToggleButtonGroup } from "@heroui/react";
import { Icon } from "@iconify/react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/api";
import { FileTree } from "../components/file-tree";
import { EditorPanel } from "../components/editor-panel";
import { useWorkspaceFiles } from "../hooks/use-workspace-files";
import type { Route } from "./+types/workspace_.$id";

const FRAMEWORK_TEMPLATES = [
  { id: "react-router", name: "React Router", icon: "lucide:route", desc: "React Router v7 with SSR" },
  { id: "nuxtjs", name: "Nuxt.js", icon: "lucide:hexagon", desc: "Full-stack Nuxt.js with SSR" },
  { id: "vite", name: "Vite", icon: "lucide:zap", desc: "Vite SPA with Worker API routes" },
  { id: "worker", name: "Worker", icon: "lucide:file-code", desc: "Simple Cloudflare Worker" },
  { id: "hono", name: "Hono", icon: "lucide:flame", desc: "Hono app on Cloudflare Workers" },
];

export function meta({ params }: Route.MetaArgs) {
  return [{ title: `Workspace ${params.id} - Fugue Dashboard` }];
}

export default function WorkspaceEditor() {
  const { id } = useParams();
  const navigate = useNavigate();
  const queryClient = useQueryClient();

  const [deployOpen, setDeployOpen] = useState(false);
  const [deploying, setDeploying] = useState(false);
  const [deployError, setDeployError] = useState<string | null>(null);
  const [deploySuccess, setDeploySuccess] = useState<string | null>(null);
  const [createNew, setCreateNew] = useState(true);
  const [newAppName, setNewAppName] = useState("");
  const [selectedAppId, setSelectedAppId] = useState<string | null>(null);
  const [renameOpen, setRenameOpen] = useState(false);
  const [renameValue, setRenameValue] = useState("");

  const { data: workspace, isLoading } = useQuery({
    queryKey: ["workspace", id],
    queryFn: () => api.getWorkspace(id!),
    enabled: !!id,
  });

  const { data: apps } = useQuery({
    queryKey: ["apps"],
    queryFn: () => api.listApps(),
  });

  const editor = useWorkspaceFiles({
    workspaceId: id!,
    initialFiles: workspace?.files ?? {},
    saveStrategy: "auto",
    autoSaveDelay: 1000,
  });

  const blocker = useBlocker(editor.dirtyFiles.length > 0);

  useEffect(() => {
    if (editor.dirtyFiles.length === 0) return;
    const handler = (e: BeforeUnloadEvent) => {
      e.preventDefault();
      e.returnValue = "";
    };
    window.addEventListener("beforeunload", handler);
    return () => window.removeEventListener("beforeunload", handler);
  }, [editor.dirtyFiles]);

  const fileTreeFiles = useMemo(
    () => Object.fromEntries(Object.values(editor.files).map((f) => [f.path, f.currentContent])),
    [editor.files],
  );

  const handleDeploy = async () => {
    if (!workspace || !id) return;
    setDeploying(true);
    setDeployError(null);
    setDeploySuccess(null);

    try {
      const filesToUpload: Record<string, string> = {};
      for (const file of Object.values(editor.files)) {
        filesToUpload[file.path] = file.currentContent;
      }

      let appId: string;

      if (createNew) {
        if (!newAppName.trim()) {
          setDeployError("App name is required");
          setDeploying(false);
          return;
        }
        const app = await api.createApp({
          name: newAppName.trim(),
          framework: workspace.framework,
          description: `Created from ${workspace.framework} template`,
        });
        appId = app.id;
      } else {
        if (!selectedAppId) {
          setDeployError("Select an app to deploy to");
          setDeploying(false);
          return;
        }
        appId = selectedAppId;
      }

      await api.deployWorkspace(id, appId);
      setDeploySuccess(appId);
      await queryClient.invalidateQueries({ queryKey: ["apps"] });
    } catch (e) {
      setDeployError(e instanceof Error ? e.message : "Deploy failed");
    } finally {
      setDeploying(false);
    }
  };

  const handleRename = async () => {
    if (!renameValue.trim() || !id) return;
    try {
      await api.updateWorkspace(id, { name: renameValue.trim() });
      await queryClient.invalidateQueries({ queryKey: ["workspace", id] });
      setRenameOpen(false);
    } catch (e) {
      console.error("Failed to rename:", e);
    }
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-screen">
        <Spinner size="lg" />
      </div>
    );
  }

  if (!workspace) {
    return (
      <div className="text-center py-20">
        <p className="text-muted">Workspace not found</p>
        <Link to="/workspace" className="text-accent mt-2 inline-block hover:underline">Back to Workspace</Link>
      </div>
    );
  }

  const frameworkLabel = FRAMEWORK_TEMPLATES.find((t) => t.id === workspace.framework)?.name ?? workspace.framework;

  return (
    <div className="flex flex-col h-screen">
      <div className="flex items-center gap-2 text-sm text-muted px-6 pt-4 pb-2 shrink-0">
        <Link to="/workspace" className="text-accent hover:underline">Workspace</Link>
        <Icon icon="lucide:chevron-right" className="w-3 h-3" />
        <span>{workspace.name}</span>
      </div>

      <div className="flex items-center justify-between px-6 pb-3 shrink-0">
        <div className="flex items-center gap-3">
          <h1 className="text-2xl font-bold">{workspace.name}</h1>
          <button
            className="text-muted hover:text-foreground transition-colors"
            onClick={() => { setRenameValue(workspace.name); setRenameOpen(true); }}
          >
            <Icon icon="lucide:pencil" className="w-3.5 h-3.5" />
          </button>
          <span className="text-xs text-muted px-2 py-0.5 rounded bg-surface-tertiary">{frameworkLabel}</span>
        </div>
        <div className="flex items-center gap-2">
          {editor.dirtyFiles.length > 0 && (
            <span className="text-xs text-warning flex items-center gap-1">
              <Icon icon="lucide:clock" className="w-3 h-3" />
              {editor.dirtyFiles.some((f) => f.status === "saving") ? "Saving..." : "Unsaved changes"}
            </span>
          )}
          <Button size="sm" onPress={() => setDeployOpen(true)}>
            <Icon icon="lucide:rocket" className="w-3 h-3" />
            Deploy
          </Button>
        </div>
      </div>

      <div className="flex gap-4 flex-1 min-h-0 px-6 pb-4">
        <div className="w-64 shrink-0 bg-surface-secondary rounded-lg p-3 overflow-auto border border-border">
          <h2 className="text-xs font-semibold text-muted uppercase tracking-wider mb-2 px-2">Files</h2>
          <FileTree
            files={fileTreeFiles}
            selectedPath={editor.activePath ?? undefined}
            onSelect={editor.openFile}
          />
        </div>
        <div className="flex-1 min-w-0 min-h-0 flex flex-col">
          <EditorPanel
            files={editor.files}
            activePath={editor.activePath}
            openPaths={editor.openPaths}
            onOpen={editor.openFile}
            onClose={editor.closeFile}
            onChange={editor.updateContent}
            onSave={editor.saveFile}
            onSaveAll={editor.saveAll}
            mode="edit"
            saveStrategy="auto"
          />
        </div>
      </div>

      <Modal.Backdrop isOpen={renameOpen} onOpenChange={setRenameOpen}>
        <Modal.Container>
          <Modal.Dialog className="sm:max-w-sm">
            <Modal.CloseTrigger />
            <Modal.Header>
              <Modal.Heading>Rename Workspace</Modal.Heading>
            </Modal.Header>
            <Modal.Body>
              <label className="block text-sm font-medium mb-1">Name</label>
              <Input
                value={renameValue}
                onChange={(e) => setRenameValue(e.target.value)}
                placeholder="workspace-name"
                variant="secondary"
                autoFocus
              />
            </Modal.Body>
            <Modal.Footer>
              <Button slot="close" variant="secondary">Cancel</Button>
              <Button onPress={handleRename} isDisabled={!renameValue.trim()}>Rename</Button>
            </Modal.Footer>
          </Modal.Dialog>
        </Modal.Container>
      </Modal.Backdrop>

      <Modal.Backdrop isOpen={deployOpen} onOpenChange={(open) => {
        if (open) {
          setDeployError(null);
          setDeploySuccess(null);
        }
        setDeployOpen(open);
      }}>
        <Modal.Container>
          <Modal.Dialog className="sm:max-w-md">
            <Modal.CloseTrigger />
            <Modal.Header>
              <Modal.Heading>Deploy</Modal.Heading>
              <p className="text-sm text-muted">Deploy your code to an application</p>
            </Modal.Header>
            <Modal.Body>
              {deploySuccess ? (
                <div className="flex flex-col items-center gap-4 py-4">
                  <div className="flex items-center justify-center w-12 h-12 rounded-full bg-success-soft">
                    <Icon icon="lucide:check" className="w-6 h-6 text-success" />
                  </div>
                  <p className="text-sm text-center">Deploy started successfully</p>
                  <Button onPress={() => { setDeployOpen(false); navigate(`/deployments/${deploySuccess}`); }}>
                    <Icon icon="lucide:external-link" className="w-4 h-4" />
                    View App
                  </Button>
                </div>
              ) : (
                <div className="space-y-4">
                  <ToggleButtonGroup
                    selectionMode="single"
                    disallowEmptySelection
                    selectedKeys={new Set([createNew ? "new" : "existing"])}
                    onSelectionChange={(keys) => {
                      const val = Array.from(keys)[0] as string;
                      setCreateNew(val === "new");
                    }}
                  >
                    <ToggleButton id="new">New App</ToggleButton>
                    <ToggleButton id="existing"><ToggleButtonGroup.Separator />Existing App</ToggleButton>
                  </ToggleButtonGroup>

                  {createNew ? (
                    <div>
                      <label className="text-sm font-medium mb-1 block">App Name</label>
                      <Input
                        value={newAppName}
                        onChange={(e) => setNewAppName(e.target.value)}
                        placeholder="my-app"
                        variant="secondary"
                      />
                      <p className="text-xs text-muted mt-1">Framework: {frameworkLabel}</p>
                    </div>
                  ) : (
                    <div>
                      <label className="text-sm font-medium mb-1 block">Select App</label>
                      {apps && apps.length > 0 ? (
                        <div className="space-y-2 max-h-48 overflow-auto">
                          {apps.map((app) => (
                            <button
                              key={app.id}
                              className={`w-full text-left p-2 rounded-lg border transition-colors ${
                                selectedAppId === app.id
                                  ? "border-accent bg-accent-soft"
                                  : "border-border hover:bg-surface-tertiary"
                              }`}
                              onClick={() => setSelectedAppId(app.id)}
                            >
                              <div className="flex items-center justify-between">
                                <span className="text-sm font-medium">{app.name}</span>
                                <span className="text-xs text-muted">{app.framework}</span>
                              </div>
                              <span className="text-xs text-muted">{app.status}</span>
                            </button>
                          ))}
                        </div>
                      ) : (
                        <p className="text-sm text-muted">No apps yet. Create one first.</p>
                      )}
                    </div>
                  )}

                  {deployError && <p className="text-sm text-danger">{deployError}</p>}
                </div>
              )}
            </Modal.Body>
            <Modal.Footer>
              {deploySuccess ? (
                <Button slot="close" variant="secondary">Close</Button>
              ) : (
                <>
                  <Button slot="close" variant="secondary">Cancel</Button>
                  <Button onPress={handleDeploy} isDisabled={deploying || (!createNew && !selectedAppId)}>
                    {deploying ? <Spinner color="current" size="sm" /> : <Icon icon="lucide:rocket" className="w-4 h-4" />}
                    {deploying ? "Deploying..." : "Deploy"}
                  </Button>
                </>
              )}
            </Modal.Footer>
          </Modal.Dialog>
        </Modal.Container>
      </Modal.Backdrop>

      {blocker.state === "blocked" && (
        <Modal.Backdrop isOpen onOpenChange={() => blocker.reset?.()}>
          <Modal.Container>
            <Modal.Dialog className="sm:max-w-sm">
              <Modal.Header>
                <Modal.Heading>Unsaved changes</Modal.Heading>
              </Modal.Header>
              <Modal.Body>
                <p className="text-sm text-muted">
                  You have {editor.dirtyFiles.length} unsaved file{editor.dirtyFiles.length === 1 ? "" : "s"}. Leaving now will discard your changes.
                </p>
              </Modal.Body>
              <Modal.Footer>
                <Button variant="secondary" onPress={() => blocker.reset?.()}>
                  Stay
                </Button>
                <Button onPress={() => blocker.proceed?.()}>
                  Leave without saving
                </Button>
              </Modal.Footer>
            </Modal.Dialog>
          </Modal.Container>
        </Modal.Backdrop>
      )}
    </div>
  );
}
