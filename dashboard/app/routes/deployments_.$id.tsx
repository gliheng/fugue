import { useState } from "react";
import { useParams, Link, useSearchParams } from "react-router";
import { Button, Card, Chip, Separator, Spinner, Tabs, Modal } from "@heroui/react";
import { Icon } from "@iconify/react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api, getAppUrl } from "../lib/api";
import { StatusBadge } from "../components/status-badge";
import { DeployButton } from "../components/deploy-button";
import { CodeEditor } from "../components/code-editor";
import { FileTree } from "../components/file-tree";
import { BuildLog } from "../components/build-log";
import type { Route } from "./+types/deployments_.$id";

export function meta({ params }: Route.MetaArgs) {
  return [{ title: `Deployment ${params.id} - Fugue Dashboard` }];
}

export default function DeploymentDetail() {
  const { id } = useParams();
  const [searchParams] = useSearchParams();
  const [activeTab, setActiveTab] = useState(searchParams.get("tab") ?? "overview");
  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const [deleteOpen, setDeleteOpen] = useState(false);
  const queryClient = useQueryClient();

  const { data: app, isLoading } = useQuery({
    queryKey: ["app", id],
    queryFn: () => api.getApp(id!),
    enabled: !!id,
  });

  const { data: source } = useQuery({
    queryKey: ["source", id],
    queryFn: () => api.getSource(id!),
    enabled: !!id && activeTab === "code",
  });

  const { data: builds } = useQuery({
    queryKey: ["builds", id],
    queryFn: () => api.getBuilds(id!),
    enabled: !!id,
  });

  const startMutation = useMutation({
    mutationFn: () => api.startApp(id!),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["app", id] }),
  });

  const stopMutation = useMutation({
    mutationFn: () => api.stopApp(id!),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["app", id] }),
  });

  const deleteMutation = useMutation({
    mutationFn: () => api.deleteApp(id!),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["apps"] });
      window.location.href = "/deployments";
    },
  });

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-20">
        <Spinner size="lg" />
      </div>
    );
  }

  if (!app) {
    return (
      <div className="text-center py-20">
        <p className="text-muted">App not found</p>
        <Link to="/deployments" className="text-accent mt-2 inline-block">Back to Deployments</Link>
      </div>
    );
  }

  const fileEntries = source?.files ?? {};
  const filePaths = Object.keys(fileEntries);
  const currentFileContent = selectedFile && fileEntries[selectedFile]
    ? fileEntries[selectedFile]
    : "";
  const frameworkLabel =
    app.framework === "react-router"
      ? "React Router"
      : app.framework === "nuxtjs"
        ? "Nuxt.js"
        : "Worker";

  return (
    <div className="p-8 max-w-6xl mx-auto">
      <div className="flex items-center gap-2 text-sm text-muted mb-4">
        <Link to="/deployments" className="text-accent hover:underline">Deployments</Link>
        <Icon icon="lucide:chevron-right" className="w-3 h-3" />
        <span>{app.name}</span>
      </div>

      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center gap-4">
          <h1 className="text-2xl font-bold">{app.name}</h1>
          <StatusBadge status={app.status} />
          <Chip size="sm" variant="outline">{frameworkLabel}</Chip>
        </div>
        <div className="flex items-center gap-2">
          {app.status === "running" ? (
            <Button variant="secondary" onPress={() => stopMutation.mutate()} isDisabled={stopMutation.isPending}>
              <Icon icon="lucide:square" className="w-4 h-4" />
              Stop
            </Button>
          ) : (
            <Button onPress={() => startMutation.mutate()} isDisabled={startMutation.isPending || app.status === "building"}>
              <Icon icon="lucide:play" className="w-4 h-4" />
              Start
            </Button>
          )}
          <DeployButton
            appId={app.id}
            hasDeploy={!!builds && builds.length > 0}
            onDeployed={() => queryClient.invalidateQueries({ queryKey: ["app", id] })}
          />
        </div>
      </div>

      <Tabs
        selectedKey={activeTab}
        onSelectionChange={(key) => setActiveTab(key as string)}
      >
        <Tabs.ListContainer>
          <Tabs.List aria-label="App details">
            <Tabs.Tab id="overview">Overview<Tabs.Indicator /></Tabs.Tab>
            <Tabs.Tab id="code">Code<Tabs.Indicator /></Tabs.Tab>
            <Tabs.Tab id="builds">Builds<Tabs.Indicator /></Tabs.Tab>
            <Tabs.Tab id="settings">Settings<Tabs.Indicator /></Tabs.Tab>
          </Tabs.List>
        </Tabs.ListContainer>

        <Tabs.Panel id="overview" className="mt-6">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <Card>
              <Card.Header>
                <Card.Title>Status</Card.Title>
              </Card.Header>
              <Card.Content>
                <div className="space-y-3">
                  <div className="flex justify-between">
                    <span className="text-sm text-muted">Status</span>
                    <StatusBadge status={app.status} />
                  </div>
                  {app.status === "running" && (
                    <div className="flex justify-between">
                      <span className="text-sm text-muted">URL</span>
                      <a
                        href={getAppUrl(app.subdomain)}
                        className="text-sm text-accent hover:underline"
                        target="_blank"
                        rel="noopener noreferrer"
                      >
                        {new URL(getAppUrl(app.subdomain)).host}
                      </a>
                    </div>
                  )}
                  <div className="flex justify-between">
                    <span className="text-sm text-muted">Framework</span>
                    <span className="text-sm">{frameworkLabel}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sm text-muted">Created</span>
                    <span className="text-sm">{new Date(app.created_at).toLocaleDateString()}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sm text-muted">Updated</span>
                    <span className="text-sm">{new Date(app.updated_at).toLocaleDateString()}</span>
                  </div>
                </div>
              </Card.Content>
            </Card>

            <Card>
              <Card.Header>
                <Card.Title>Recent Builds</Card.Title>
              </Card.Header>
              <Card.Content>
                {builds && builds.length > 0 ? (
                  <div className="space-y-2">
                    {builds.slice(0, 5).map((build) => (
                      <div key={build.id} className="flex items-center justify-between py-2 border-b border-border last:border-0">
                        <div className="flex items-center gap-2">
                          <Chip
                            color={
                              build.status === "success" ? "success"
                              : build.status === "failed" ? "danger"
                              : "warning"
                            }
                            size="sm"
                            variant="soft"
                          >
                            {build.status}
                          </Chip>
                          <span className="text-xs text-muted">
                            {new Date(build.created_at).toLocaleString()}
                          </span>
                        </div>
                        {build.error && (
                          <span className="text-xs text-danger truncate max-w-40">{build.error}</span>
                        )}
                      </div>
                    ))}
                  </div>
                ) : (
                  <p className="text-sm text-muted text-center py-4">No builds yet</p>
                )}
              </Card.Content>
            </Card>

            {app.description && (
              <Card className="md:col-span-2">
                <Card.Header>
                  <Card.Title>Description</Card.Title>
                </Card.Header>
                <Card.Content>
                  <p className="text-sm text-muted">{app.description}</p>
                </Card.Content>
              </Card>
            )}
          </div>
        </Tabs.Panel>

        <Tabs.Panel id="code" className="mt-6">
          {filePaths.length > 0 ? (
            <div className="flex gap-4 h-[600px]">
              <div className="w-64 shrink-0 bg-surface-secondary rounded-lg p-3 overflow-auto">
                <FileTree
                  files={fileEntries}
                  selectedPath={selectedFile ?? undefined}
                  onSelect={setSelectedFile}
                />
              </div>
              <div className="flex-1 min-w-0">
                {selectedFile && fileEntries[selectedFile] !== undefined ? (
                  <CodeEditor
                    appId={app.id}
                    filePath={selectedFile}
                    content={fileEntries[selectedFile]}
                    readOnly
                  />
                ) : (
                  <div className="flex items-center justify-center h-full text-muted">
                    Select a file to view or edit
                  </div>
                )}
              </div>
            </div>
          ) : (
            <div className="text-center py-20">
              <Icon icon="lucide:folder-open" className="w-12 h-12 text-muted mx-auto mb-4" />
              <p className="text-muted mb-4">No source files uploaded yet</p>
              <p className="text-sm text-muted">Use the workspace to create and deploy code</p>
            </div>
          )}
        </Tabs.Panel>

        <Tabs.Panel id="builds" className="mt-6">
          {builds && builds.length > 0 ? (
            <div className="space-y-4">
              {builds.map((build) => (
                <Card key={build.id}>
                  <Card.Header className="flex items-center justify-between">
                    <div className="flex items-center gap-3">
                      <Chip
                        color={
                          build.status === "success" ? "success"
                          : build.status === "failed" ? "danger"
                          : "warning"
                        }
                        size="sm"
                        variant="soft"
                      >
                        {build.status}
                      </Chip>
                      <span className="text-sm text-muted">
                        {new Date(build.created_at).toLocaleString()}
                      </span>
                    </div>
                    {build.status === "running" && (
                      <Link to={`/deployments/${id}/deploy`}>
                        <Button size="sm" variant="ghost">View Logs</Button>
                      </Link>
                    )}
                  </Card.Header>
                  {build.log && (
                    <Card.Content>
                      <pre className="text-xs font-mono bg-black text-green-400 p-3 rounded-lg overflow-auto max-h-48">
                        {build.log}
                      </pre>
                    </Card.Content>
                  )}
                  {build.error && (
                    <Card.Content>
                      <p className="text-sm text-danger">{build.error}</p>
                    </Card.Content>
                  )}
                </Card>
              ))}
            </div>
          ) : (
            <div className="text-center py-20">
              <Icon icon="lucide:hammer" className="w-12 h-12 text-muted mx-auto mb-4" />
              <p className="text-muted">No builds yet</p>
            </div>
          )}
        </Tabs.Panel>

        <Tabs.Panel id="settings" className="mt-6">
          <div className="space-y-6">
            <Card>
              <Card.Header>
                <Card.Title>Environment Variables</Card.Title>
                <Card.Description>Configure environment variables for this app</Card.Description>
              </Card.Header>
              <Card.Content>
                {Object.keys(app.env_vars || {}).length > 0 ? (
                  <div className="space-y-2">
                    {Object.entries(app.env_vars || {}).map(([key, value]) => (
                      <div key={key} className="flex items-center gap-2 font-mono text-sm">
                        <span className="text-accent">{key}</span>
                        <span className="text-muted">=</span>
                        <span>{String(value)}</span>
                      </div>
                    ))}
                  </div>
                ) : (
                  <p className="text-sm text-muted">No environment variables set</p>
                )}
              </Card.Content>
            </Card>

            <Card variant="danger">
              <Card.Header>
                <Card.Title>Danger Zone</Card.Title>
              </Card.Header>
              <Card.Content>
                <p className="text-sm text-muted mb-4">
                  Permanently delete this application and all its data. This action cannot be undone.
                </p>
                <Button variant="danger" onPress={() => setDeleteOpen(true)}>
                  <Icon icon="lucide:trash-2" className="w-4 h-4" />
                  Delete App
                </Button>
              </Card.Content>
            </Card>
          </div>

          <Modal.Backdrop isOpen={deleteOpen} onOpenChange={setDeleteOpen}>
            <Modal.Container>
              <Modal.Dialog className="sm:max-w-[360px]">
                <Modal.Header>
                  <Modal.Icon className="bg-danger-soft text-danger-soft-foreground">
                    <Icon icon="lucide:trash-2" className="size-5" />
                  </Modal.Icon>
                  <Modal.Heading>Delete {app.name}?</Modal.Heading>
                </Modal.Header>
                <Modal.Body>
                  <p className="text-sm text-muted">
                    This will permanently delete the app and all its data. This action cannot be undone.
                  </p>
                </Modal.Body>
                <Modal.Footer>
                  <Button slot="close" variant="secondary">
                    Cancel
                  </Button>
                  <Button variant="danger" onPress={() => deleteMutation.mutate()} isDisabled={deleteMutation.isPending}>
                    {deleteMutation.isPending ? "Deleting..." : "Delete"}
                  </Button>
                </Modal.Footer>
              </Modal.Dialog>
            </Modal.Container>
          </Modal.Backdrop>
        </Tabs.Panel>
      </Tabs>
    </div>
  );
}
