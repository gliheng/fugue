import { useCallback, useEffect, useState } from "react";
import { Button, Spinner } from "@heroui/react";
import { Icon } from "@iconify/react";
import { api, getAppUrl } from "../lib/api";

interface PreviewFrameProps {
  appId: string;
  appUrl: string;
  onRefresh?: () => void;
}

export function PreviewFrame({ appId, appUrl, onRefresh }: PreviewFrameProps) {
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const checkStatus = async () => {
      try {
        const status = await api.getAppStatus(appId);
        if (status.status === "running" && status.url) {
          setLoading(false);
        } else if (status.status === "error") {
          setError("App failed to start");
          setLoading(false);
        }
      } catch {
        // Still deploying, keep polling
      }
    };

    const interval = setInterval(checkStatus, 2000);
    checkStatus();

    // Give up after 60s
    const timeout = setTimeout(() => {
      if (loading) {
        setError("App startup timed out");
        setLoading(false);
      }
    }, 60000);

    return () => {
      clearInterval(interval);
      clearTimeout(timeout);
    };
  }, [appId, loading]);

  const handleRefresh = useCallback(() => {
    setLoading(true);
    setError(null);
    onRefresh?.();
  }, [onRefresh]);

  const handleOpenExternal = useCallback(() => {
    window.open(appUrl, "_blank");
  }, [appUrl]);

  if (loading) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-4">
        <Spinner size="lg" />
        <p className="text-sm text-muted">Starting app preview...</p>
        <p className="text-xs text-muted">This may take a moment</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-4">
        <Icon icon="lucide:alert-triangle" className="w-12 h-12 text-warning" />
        <p className="text-sm text-foreground">{error}</p>
        <Button size="sm" variant="secondary" onPress={handleRefresh}>
          <Icon icon="lucide:refresh-cw" className="w-3 h-3" />
          Retry
        </Button>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center gap-2 px-4 py-2 border-b border-border bg-surface-secondary">
        <div className="flex-1 flex items-center gap-2 text-xs text-muted font-mono truncate">
          <Icon icon="lucide:globe" className="w-3 h-3 shrink-0" />
          <span className="truncate">{appUrl}</span>
        </div>
        <button
          className="text-muted hover:text-foreground transition-colors"
          onClick={handleRefresh}
          title="Refresh"
        >
          <Icon icon="lucide:refresh-cw" className="w-3.5 h-3.5" />
        </button>
        <button
          className="text-muted hover:text-foreground transition-colors"
          onClick={handleOpenExternal}
          title="Open in new tab"
        >
          <Icon icon="lucide:external-link" className="w-3.5 h-3.5" />
        </button>
      </div>
      <iframe
        src={appUrl}
        className="flex-1 w-full border-0 bg-white"
        title="App Preview"
        sandbox="allow-scripts allow-same-origin allow-forms allow-popups"
      />
    </div>
  );
}
