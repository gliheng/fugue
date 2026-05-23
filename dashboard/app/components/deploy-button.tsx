import { Button, Spinner } from "@heroui/react";
import { Icon } from "@iconify/react";
import { useState } from "react";
import { api } from "../lib/api";

export function DeployButton({
  appId,
  hasDeploy,
  onDeployed,
}: {
  appId: string;
  hasDeploy: boolean;
  onDeployed: (data: { build_id: string; status: string }) => void;
}) {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleDeploy = async () => {
    setLoading(true);
    setError(null);
    try {
      let data;
      if (hasDeploy) {
        data = await api.redeploy(appId);
      } else {
        data = await api.deploy(appId);
      }
      onDeployed(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Deploy failed");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="flex items-center gap-3">
      <Button
        color="accent"
        onPress={handleDeploy}
        isDisabled={loading}
      >
        {loading && <Spinner color="current" size="sm" />}
        <Icon icon="lucide:rocket" className="w-4 h-4" />
        {loading ? "Deploying..." : hasDeploy ? "Redeploy" : "Deploy"}
      </Button>
      {error && <span className="text-sm text-danger">{error}</span>}
    </div>
  );
}
