import { Suspense, lazy, useCallback, useEffect, useRef, useState } from "react";
import { useParams, Link } from "react-router";
import { Button, Card, Input, Modal, Spinner, ToggleButton, ToggleButtonGroup } from "@heroui/react";
import { Icon } from "@iconify/react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "react-router";
import { api } from "../lib/api";
import { FileTree } from "../components/file-tree";
import type { Route } from "./+types/workspace_.$id";

const FRAMEWORK_TEMPLATES = [
  { id: "react-router", name: "React Router", icon: "lucide:route", desc: "React Router v7 with SSR" },
  { id: "nuxtjs", name: "Nuxt.js", icon: "lucide:hexagon", desc: "Full-stack Nuxt.js with SSR" },
  { id: "worker", name: "Worker", icon: "lucide:file-code", desc: "Simple Cloudflare Worker" },
];

export function meta({ params }: Route.MetaArgs) {
  return [{ title: `Workspace ${params.id} - Fugue Dashboard` }];
}

export default function WorkspaceEditor() {
  const { id } = useParams();
  const navigate = useNavigate();
  const queryClient = useQueryClient();

  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const [localEdits, setLocalEdits] = useState<Record<string, string>>({});
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

  const updateMutation = useMutation({
    mutationFn: (files: Record<string, string>) =>
      api.updateWorkspace(id!, { files }),
  });

  const saveTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const handleEdit = useCallback((filePath: string, content: string) => {
    setLocalEdits((prev) => ({ ...prev, [filePath]: content }));

    if (saveTimeoutRef.current) clearTimeout(saveTimeoutRef.current);
    saveTimeoutRef.current = setTimeout(() => {
      const merged = { ...(workspace?.files ?? {}), ...localEdits, [filePath]: content };
      updateMutation.mutate(merged);
    }, 1000);
  }, [id, workspace, localEdits, updateMutation]);

  useEffect(() => {
    return () => {
      if (saveTimeoutRef.current) clearTimeout(saveTimeoutRef.current);
    };
  }, []);

  const files = { ...(workspace?.files ?? {}), ...localEdits };
  const fileList = Object.keys(workspace?.files ?? {}).sort();

  const handleDeploy = async () => {
    if (!workspace) return;
    setDeploying(true);
    setDeployError(null);
    setDeploySuccess(null);

    try {
      const sourceToUpload = files;
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

      await api.uploadSourceFiles(appId, sourceToUpload);
      await api.deploy(appId);
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
      <div className="flex items-center justify-center py-20">
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
    <div className="p-8 max-w-6xl mx-auto">
      <div className="flex items-center gap-2 text-sm text-muted mb-4">
        <Link to="/workspace" className="text-accent hover:underline">Workspace</Link>
        <Icon icon="lucide:chevron-right" className="w-3 h-3" />
        <span>{workspace.name}</span>
      </div>

      <div className="flex items-center justify-between mb-4">
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
          <Button size="sm" onPress={() => setDeployOpen(true)}>
            <Icon icon="lucide:rocket" className="w-3 h-3" />
            Deploy
          </Button>
        </div>
      </div>

      {Object.keys(localEdits).length > 0 && (
        <p className="text-xs text-warning mb-2">
          <Icon icon="lucide:clock" className="w-3 h-3 inline mr-1" />
          Auto-saving changes...
        </p>
      )}

      <div className="flex gap-4 h-[600px]">
        <div className="w-64 shrink-0 bg-surface-secondary rounded-lg p-3 overflow-auto">
          <FileTree
            files={files}
            selectedPath={selectedFile ? `/${selectedFile}` : undefined}
            onSelect={(path) => setSelectedFile(path.startsWith("/") ? path.slice(1) : path)}
          />
        </div>
        <div className="flex-1 min-w-0">
          {selectedFile && files[selectedFile] !== undefined ? (
            <EditorPanel
              filePath={selectedFile}
              content={files[selectedFile]}
              onChange={(content) => handleEdit(selectedFile, content)}
            />
          ) : (
            <div className="flex items-center justify-center h-full text-muted">
              Select a file to edit
            </div>
          )}
        </div>
      </div>

      <Modal>
        <Modal.Backdrop isOpen={renameOpen} onOpenChange={setRenameOpen}>
          <Modal.Container>
            <Modal.Dialog className="sm:max-w-sm">
              <Modal.CloseTrigger />
              <Modal.Header>
                <Modal.Heading>Rename Workspace</Modal.Heading>
              </Modal.Header>
              <Modal.Body>
                <Input
                  value={renameValue}
                  onChange={(e) => setRenameValue(e.target.value)}
                  placeholder="workspace-name"
                  variant="secondary"
                />
              </Modal.Body>
              <Modal.Footer>
                <Button slot="close" variant="secondary">Cancel</Button>
                <Button onPress={handleRename} isDisabled={!renameValue.trim()}>Rename</Button>
              </Modal.Footer>
            </Modal.Dialog>
          </Modal.Container>
        </Modal.Backdrop>
      </Modal>

      <Modal>
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
      </Modal>
    </div>
  );
}

function EditorPanel({
  filePath,
  content,
  onChange,
}: {
  filePath: string;
  content: string;
  onChange: (content: string) => void;
}) {
  const language = getLanguage(filePath);

  return (
    <Card className="w-full h-full">
      <Card.Header className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Icon icon="lucide:file-code" className="w-4 h-4 text-accent" />
          <span className="text-sm font-mono text-muted">{filePath}</span>
        </div>
      </Card.Header>
      <Card.Content className="p-0 overflow-hidden">
        <Suspense
          fallback={
            <div className="flex items-center justify-center h-96">
              <Spinner size="lg" />
            </div>
          }
        >
          <MonacoEditor
            height="500px"
            language={language}
            value={content}
            onChange={(v) => onChange(v ?? "")}
            theme="vs-dark"
            options={{
              minimap: { enabled: false },
              fontSize: 13,
              lineNumbers: "on",
              wordWrap: "on",
              padding: { top: 8 },
            }}
          />
        </Suspense>
      </Card.Content>
    </Card>
  );
}

function getLanguage(path: string): string {
  const ext = path.split(".").pop()?.toLowerCase() ?? "";
  const map: Record<string, string> = {
    ts: "typescript", tsx: "typescriptreact", js: "javascript", jsx: "javascript",
    json: "json", css: "css", html: "html", md: "markdown", toml: "toml",
    rs: "rust", yml: "yaml", yaml: "yaml",
  };
  return map[ext] ?? "plaintext";
}