"use client";

import { Suspense, lazy, useState } from "react";
import { Button, Card, Spinner } from "@heroui/react";
import { Icon } from "@iconify/react";
import { api } from "../lib/api";

const MonacoEditor = lazy(() => import("@monaco-editor/react"));

export function CodeEditor({
  appId,
  filePath,
  content: initialContent,
  onSave,
}: {
  appId: string;
  filePath: string;
  content: string;
  onSave?: () => void;
}) {
  const [content, setContent] = useState(initialContent);
  const [saving, setSaving] = useState(false);
  const [dirty, setDirty] = useState(false);

  const handleChange = (value: string | undefined) => {
    if (value !== undefined && value !== content) {
      setContent(value);
      setDirty(true);
    }
  };

  const handleSave = async () => {
    setSaving(true);
    try {
      await api.updateSource(appId, filePath, content);
      setDirty(false);
      onSave?.();
    } catch (e) {
      console.error("Failed to save:", e);
    } finally {
      setSaving(false);
    }
  };

  const language = getLanguage(filePath);

  return (
    <Card className="w-full h-full">
      <Card.Header className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Icon icon="lucide:file-code" className="w-4 h-4 text-accent" />
          <span className="text-sm font-mono text-muted">{filePath}</span>
        </div>
        <div className="flex items-center gap-2">
          {dirty && <span className="text-xs text-warning">Unsaved changes</span>}
          <Button size="sm" variant="secondary" onPress={handleSave} isDisabled={!dirty || saving}>
            {saving ? <Spinner color="current" size="sm" /> : <Icon icon="lucide:save" className="w-3 h-3" />}
            Save
          </Button>
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
            onChange={handleChange}
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
    ts: "typescript",
    tsx: "typescriptreact",
    js: "javascript",
    jsx: "javascript",
    json: "json",
    css: "css",
    html: "html",
    md: "markdown",
    toml: "toml",
    rs: "rust",
    yml: "yaml",
    yaml: "yaml",
  };
  return map[ext] ?? "plaintext";
}
