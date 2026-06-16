"use client";

import { useCallback, useEffect, useState } from "react";
import { Button, Card, Modal, Spinner } from "@heroui/react";
import { Icon } from "@iconify/react";
import { CodeEditor, type EditorMode } from "./code-editor";
import { getFileIcon } from "../lib/icons";
import type { FileState } from "../lib/file-state";
import type { SaveStrategy } from "../hooks/use-workspace-files";

export interface EditorPanelProps {
  files: Record<string, FileState>;
  activePath: string | null;
  openPaths: string[];
  onOpen: (path: string) => void;
  onClose: (path: string) => void;
  onChange: (path: string, content: string) => void;
  onSave: (path: string) => void;
  onSaveAll: () => void;
  mode?: EditorMode;
  saveStrategy?: SaveStrategy;
}

export function EditorPanel({
  files,
  activePath,
  openPaths,
  onOpen,
  onClose,
  onChange,
  onSave,
  onSaveAll,
  mode = "edit",
  saveStrategy = "auto",
}: EditorPanelProps) {
  const activeFile = activePath ? files[activePath] : null;
  const [closingPath, setClosingPath] = useState<string | null>(null);

  const anyDirty = Object.values(files).some((f) => f.status === "dirty" || f.status === "error");

  const handleClose = useCallback(
    (path: string) => {
      const file = files[path];
      if (file && file.status === "dirty") {
        setClosingPath(path);
        return;
      }
      onClose(path);
    },
    [files, onClose],
  );

  const confirmClose = useCallback(() => {
    if (closingPath) {
      onClose(closingPath);
      setClosingPath(null);
    }
  }, [closingPath, onClose]);

  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      const modifier = e.metaKey || e.ctrlKey;
      if (!modifier || !activePath) return;

      // Don't intercept shortcuts while typing in inputs, textareas, or contenteditable.
      const target = e.target;
      if (target instanceof HTMLElement && target.matches("input, textarea, [contenteditable='true'], [contenteditable]")) return;

      if (e.key.toLowerCase() === "s") {
        e.preventDefault();
        onSave(activePath);
        return;
      }

      if (e.key.toLowerCase() === "w") {
        e.preventDefault();
        handleClose(activePath);
        return;
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [activePath, onSave, handleClose]);

  return (
    <Card className="w-full flex-1 flex flex-col overflow-hidden">
      {openPaths.length > 0 && (
        <div className="flex items-center gap-1 px-2 pt-2 pb-1 border-b border-border bg-surface-secondary/50 overflow-x-auto">
          {openPaths.map((path) => {
            const file = files[path];
            if (!file) return null;
            const isActive = path === activePath;
            const fileName = file.path.split("/").pop() ?? file.path;
            return (
              <div
                key={path}
                className={`group flex items-center gap-1 px-3 py-1.5 rounded-t-md text-xs border-t border-l border-r transition-colors min-w-0 ${
                  isActive
                    ? "bg-surface-primary text-foreground border-border"
                    : "bg-surface-secondary text-muted border-transparent hover:text-foreground"
                }`}
              >
                <button
                  type="button"
                  onClick={() => onOpen(path)}
                  className="flex items-center gap-2 min-w-0 bg-transparent border-0 p-0 font-inherit"
                >
                  <Icon icon={getFileIcon(fileName)} className="w-3 h-3 shrink-0" />
                  <span className="truncate max-w-[140px]">{fileName}</span>
                  {file.status === "dirty" && (
                    <span className="w-1.5 h-1.5 rounded-full bg-warning shrink-0" aria-hidden="true" />
                  )}
                  {file.status === "saving" && (
                    <Spinner color="current" size="sm" className="w-3 h-3 shrink-0" />
                  )}
                  {file.status === "error" && (
                    <span className="w-1.5 h-1.5 rounded-full bg-danger shrink-0" aria-hidden="true" />
                  )}
                </button>
                <button
                  type="button"
                  onClick={(e) => {
                    e.stopPropagation();
                    handleClose(path);
                  }}
                  className="opacity-0 group-hover:opacity-100 hover:text-danger transition-opacity shrink-0 bg-transparent border-0 p-0.5 rounded-sm"
                  aria-label={`Close ${fileName}`}
                >
                  <Icon icon="lucide:x" className="w-3 h-3" />
                </button>
              </div>
            );
          })}
        </div>
      )}

      <Card.Header className="flex items-center justify-between shrink-0 border-b border-border">
        <div className="flex items-center gap-2 min-w-0">
          <Icon icon="lucide:file-code" className="w-4 h-4 text-accent shrink-0" />
          <span className="text-sm font-mono text-muted truncate">
            {activeFile?.path ?? "No file selected"}
          </span>
        </div>
        <div className="flex items-center gap-2 shrink-0">
          {activeFile?.status === "error" && (
            <div className="flex items-center gap-2">
              <span className="text-xs text-danger truncate max-w-[200px]" title={activeFile.error}>
                {activeFile.error}
              </span>
              <Button size="sm" variant="secondary" onPress={() => activeFile && onSave(activeFile.path)}>
                Retry
              </Button>
            </div>
          )}
          {activeFile?.status === "dirty" && saveStrategy === "auto" && (
            <span className="text-xs text-warning">Unsaved changes</span>
          )}
          {activeFile?.status === "saving" && (
            <span className="text-xs text-muted flex items-center gap-1">
              <Spinner color="current" size="sm" />
              Saving…
            </span>
          )}
          {mode === "edit" && (
            <>
              <Button
                size="sm"
                variant="secondary"
                onPress={() => activePath && onSave(activePath)}
                isDisabled={!activeFile || activeFile.status !== "dirty"}
              >
                <Icon icon="lucide:save" className="w-3 h-3" />
                Save
              </Button>
              {anyDirty && (
                <Button size="sm" variant="secondary" onPress={onSaveAll}>
                  Save all
                </Button>
              )}
            </>
          )}
        </div>
      </Card.Header>

      <Card.Content className="p-0 overflow-hidden flex-1 min-h-0">
        {activeFile ? (
          <CodeEditor
            filePath={activeFile.path}
            value={activeFile.currentContent}
            mode={mode}
            openPaths={openPaths}
            onChange={(content) => onChange(activeFile.path, content)}
          />
        ) : (
          <div className="flex flex-col items-center justify-center h-full text-muted">
            <Icon icon="lucide:file-code" className="w-12 h-12 mb-3 opacity-50" />
            <p className="text-sm">Select a file from the sidebar to edit</p>
          </div>
        )}
      </Card.Content>

      <Modal.Backdrop isOpen={!!closingPath} onOpenChange={(open) => !open && setClosingPath(null)}>
        <Modal.Container>
          <Modal.Dialog className="sm:max-w-sm">
            <Modal.CloseTrigger />
            <Modal.Header>
              <Modal.Heading>Unsaved changes</Modal.Heading>
            </Modal.Header>
            <Modal.Body>
              <p className="text-sm text-muted">
                This file has unsaved changes. Are you sure you want to close it?
              </p>
            </Modal.Body>
            <Modal.Footer>
              <Button slot="close" variant="secondary">
                Keep editing
              </Button>
              <Button onPress={confirmClose}>Close without saving</Button>
            </Modal.Footer>
          </Modal.Dialog>
        </Modal.Container>
      </Modal.Backdrop>
    </Card>
  );
}
