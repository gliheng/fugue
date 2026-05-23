import { useEffect, useRef, useState } from "react";
import { Card, Spinner } from "@heroui/react";
import { BuildLogStream } from "../lib/ws";

interface LogEntry {
  timestamp: string;
  message: string;
  level?: string;
}

export function BuildLog({
  appId,
  buildId,
}: {
  appId: string;
  buildId?: string;
}) {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [connected, setConnected] = useState(false);
  const logRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const stream = new BuildLogStream(appId, buildId);
    const unsubscribe = stream.onMessage((data) => {
      if (typeof data === "string") {
        setLogs((prev) => [...prev, { timestamp: new Date().toISOString(), message: data }]);
      } else if (typeof data === "object" && data !== null) {
        const entry = data as { message?: string; timestamp?: string; level?: string };
        setLogs((prev) => [
          ...prev,
          {
            timestamp: entry.timestamp ?? new Date().toISOString(),
            message: entry.message ?? JSON.stringify(data),
            level: entry.level,
          },
        ]);
      }
    });

    stream.connect();
    setConnected(true);

    return () => {
      stream.disconnect();
      unsubscribe();
      setConnected(false);
    };
  }, [appId, buildId]);

  useEffect(() => {
    if (logRef.current) {
      logRef.current.scrollTop = logRef.current.scrollHeight;
    }
  }, [logs]);

  return (
    <Card className="w-full">
      <Card.Header className="flex items-center justify-between">
        <Card.Title className="text-sm">Build Log</Card.Title>
        <div className="flex items-center gap-2">
          <span className={`w-2 h-2 rounded-full ${connected ? "bg-success" : "bg-muted"}`} />
          <span className="text-xs text-muted">{connected ? "Live" : "Disconnected"}</span>
        </div>
      </Card.Header>
      <Card.Content>
        <div
          ref={logRef}
          className="bg-black text-green-400 font-mono text-xs p-4 rounded-lg h-80 overflow-y-auto"
        >
          {logs.length === 0 ? (
            <div className="flex items-center justify-center h-full text-muted">
              <Spinner size="sm" />
              <span className="ml-2">Waiting for build output...</span>
            </div>
          ) : (
            logs.map((log, i) => (
              <div key={i} className="leading-5">
                <span className="text-gray-500 mr-2">
                  {new Date(log.timestamp).toLocaleTimeString()}
                </span>
                <span className={log.level === "error" ? "text-red-400" : ""}>{log.message}</span>
              </div>
            ))
          )}
        </div>
      </Card.Content>
    </Card>
  );
}
