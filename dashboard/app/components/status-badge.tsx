import { Chip } from "@heroui/react";

const statusConfig: Record<string, { color: "success" | "warning" | "danger" | "default" | "accent"; label: string }> = {
  running: { color: "success", label: "Running" },
  building: { color: "warning", label: "Building" },
  deploying: { color: "accent", label: "Deploying" },
  stopped: { color: "default", label: "Stopped" },
  created: { color: "default", label: "Created" },
  error: { color: "danger", label: "Error" },
};

export function StatusBadge({ status }: { status: string }) {
  const config = statusConfig[status] ?? { color: "default" as const, label: status };
  return (
    <Chip color={config.color} size="sm" variant="soft">
      {config.label}
    </Chip>
  );
}
