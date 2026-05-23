import { useState, useRef } from "react";
import { useParams, Link, useNavigate } from "react-router";
import { Button, Card, Spinner } from "@heroui/react";
import { Icon } from "@iconify/react";
import { useQuery } from "@tanstack/react-query";
import { api, getAppUrl } from "../lib/api";
import { BuildLog } from "../components/build-log";
import type { Route } from "./+types/apps_.$id.deploy";

export function meta({ params }: Route.MetaArgs) {
  return [{ title: `Deploy - ${params.id} - Fugue Dashboard` }];
}

export default function AppDeploy() {
  const { id } = useParams();
  const navigate = useNavigate();
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [deploying, setDeploying] = useState(false);
  const [buildId, setBuildId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const { data: app } = useQuery({
    queryKey: ["app", id],
    queryFn: () => api.getApp(id!),
    enabled: !!id,
  });

  const handleDeploy = async (source?: Record<string, string>) => {
    setDeploying(true);
    setError(null);
    try {
      const result = await api.deploy(id!, source);
      setBuildId(result.build_id);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Deploy failed");
    } finally {
      setDeploying(false);
    }
  };

  const handleFileUpload = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    const formData = new FormData();
    formData.append("file", file);

    try {
      const res = await fetch(`/api/v1/apps/${id}/source`, {
        method: "POST",
        body: formData,
      });
      if (!res.ok) {
        const body = await res.json().catch(() => ({}));
        throw new Error(body.error || `Upload failed: ${res.status}`);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : "Upload failed");
    }
  };

  return (
    <div className="p-8 max-w-4xl mx-auto">
      <div className="flex items-center gap-2 text-sm text-muted mb-4">
        <Link to="/apps" className="text-accent hover:underline">Apps</Link>
        <Icon icon="lucide:chevron-right" className="w-3 h-3" />
        <Link to={`/apps/${id}`} className="text-accent hover:underline">{app?.name ?? id}</Link>
        <Icon icon="lucide:chevron-right" className="w-3 h-3" />
        <span>Deploy</span>
      </div>

      <h1 className="text-2xl font-bold mb-6">Deploy {app?.name ?? "App"}</h1>

      {error && (
        <Card className="mb-6 border-danger">
          <Card.Content>
            <div className="flex items-center gap-2 text-danger">
              <Icon icon="lucide:alert-circle" className="w-4 h-4" />
              <span className="text-sm">{error}</span>
            </div>
          </Card.Content>
        </Card>
      )}

      {buildId ? (
        <div className="space-y-6">
          <Card>
            <Card.Header>
              <Card.Title>Build in Progress</Card.Title>
              <Card.Description>Build ID: {buildId}</Card.Description>
            </Card.Header>
            <Card.Content>
              <div className="flex items-center gap-3 mb-4">
                <Spinner size="sm" />
                <span className="text-sm">Building and deploying your application...</span>
              </div>
            </Card.Content>
          </Card>

          <BuildLog appId={id!} buildId={buildId} />

          <div className="flex gap-3">
            <Button variant="secondary" onPress={() => navigate(`/apps/${id}`)}>
              View App Details
            </Button>
            {app?.status === "running" && (
              <a
                href={getAppUrl(app.subdomain)}
                target="_blank"
                rel="noopener noreferrer"
              >
                <Button>
                  <Icon icon="lucide:external-link" className="w-4 h-4" />
                  Open App
                </Button>
              </a>
            )}
          </div>
        </div>
      ) : (
        <div className="space-y-6">
          <Card>
            <Card.Header>
              <Card.Title>Upload Source</Card.Title>
              <Card.Description>
                Upload a zip file containing your application source code
              </Card.Description>
            </Card.Header>
            <Card.Content>
              <div
                className="border-2 border-dashed border-border rounded-lg p-8 text-center cursor-pointer hover:border-accent transition-colors"
                onClick={() => fileInputRef.current?.click()}
              >
                <Icon icon="lucide:upload" className="w-8 h-8 text-muted mx-auto mb-3" />
                <p className="text-sm font-medium">Click to upload a zip file</p>
                <p className="text-xs text-muted mt-1">Supports .zip archives</p>
                <input
                  ref={fileInputRef}
                  type="file"
                  accept=".zip"
                  className="hidden"
                  onChange={handleFileUpload}
                />
              </div>
            </Card.Content>
          </Card>

          <div className="flex items-center gap-4">
            <div className="flex-1 border-t border-border" />
            <span className="text-xs text-muted">OR</span>
            <div className="flex-1 border-t border-border" />
          </div>

          <Card>
            <Card.Header>
              <Card.Title>Deploy Existing Source</Card.Title>
              <Card.Description>
                Rebuild and deploy from the current source code
              </Card.Description>
            </Card.Header>
            <Card.Footer>
              <Button
                onPress={() => handleDeploy()}
                isDisabled={deploying || !app?.source_path}
              >
                {deploying ? <Spinner color="current" size="sm" /> : <Icon icon="lucide:rocket" className="w-4 h-4" />}
                {deploying ? "Deploying..." : "Deploy"}
              </Button>
              {!app?.source_path && (
                <span className="text-xs text-warning ml-2">No source code uploaded</span>
              )}
            </Card.Footer>
          </Card>
        </div>
      )}
    </div>
  );
}
