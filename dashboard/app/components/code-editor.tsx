"use client";

import { useEffect, useRef } from "react";
import { Editor, type Monaco, type OnMount } from "@monaco-editor/react";
import { useDashboardTheme, toMonacoTheme } from "../lib/theme";

export type EditorMode = "edit" | "readonly";

export interface CodeEditorProps {
  filePath: string;
  value: string;
  mode?: EditorMode;
  openPaths?: string[];
  onChange?: (value: string) => void;
  onMount?: (editor: Parameters<OnMount>[0], monaco: Monaco) => void;
}

function getLanguage(path: string): string {
  const ext = path.split(".").pop()?.toLowerCase() ?? "";
  const map: Record<string, string> = {
    ts: "typescript",
    tsx: "typescriptreact",
    js: "javascript",
    jsx: "javascriptreact",
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

export function CodeEditor({
  filePath,
  value,
  mode = "edit",
  openPaths,
  onChange,
  onMount,
}: CodeEditorProps) {
  const theme = useDashboardTheme();
  const monacoRef = useRef<Monaco | null>(null);
  const editorRef = useRef<Parameters<OnMount>[0] | null>(null);
  const activeModelsRef = useRef<Map<string, ReturnType<Monaco["editor"]["createModel"]>>>(new Map());
  const cachedModelsRef = useRef<Map<string, ReturnType<Monaco["editor"]["createModel"]>>>(new Map());
  const pendingValueRef = useRef<string | null>(null);
  const filePathRef = useRef(filePath);
  const onChangeRef = useRef(onChange);

  filePathRef.current = filePath;
  onChangeRef.current = onChange;

  const MAX_CACHED_MODELS = 10;

  const getOrCreateModel = (monaco: Monaco, path: string, content: string) => {
    const uri = monaco.Uri.file(path);
    let model = monaco.editor.getModel(uri);
    if (!model) {
      const cached = cachedModelsRef.current.get(path);
      if (cached) {
        model = cached;
        cachedModelsRef.current.delete(path);
      } else {
        model = monaco.editor.createModel(content, getLanguage(path), uri);
      }
    }
    activeModelsRef.current.set(path, model);
    return model;
  };

  const handleMount: OnMount = (editor, monaco) => {
    editorRef.current = editor;
    monacoRef.current = monaco;

    // @monaco-editor/react may have created a default model from defaultValue.
    // We manage our own models per filePath, so switch to ours and dispose the default.
    const defaultModel = editor.getModel();
    const model = getOrCreateModel(monaco, filePathRef.current, value);
    editor.setModel(model);
    if (defaultModel && defaultModel !== model) {
      defaultModel.dispose();
    }

    const disposable = editor.onDidChangeModelContent(() => {
      const value = editor.getValue();
      if (value === pendingValueRef.current) {
        pendingValueRef.current = null;
        return;
      }
      onChangeRef.current?.(value);
    });

    onMount?.(editor, monaco);

    return () => {
      disposable.dispose();
    };
  };

  useEffect(() => {
    const editor = editorRef.current;
    const monaco = monacoRef.current;
    if (!editor || !monaco) return;

    const model = getOrCreateModel(monaco, filePath, value);
    const currentModel = editor.getModel();
    if (currentModel !== model) {
      editor.setModel(model);
    }

    if (model.getValue() !== value) {
      pendingValueRef.current = value;
      model.setValue(value);
    }
  }, [filePath, value]);

  useEffect(() => {
    if (!openPaths) return;
    const openSet = new Set(openPaths);

    // Move closed-tab models to an LRU cache so undo history survives reopening.
    for (const [path, model] of Array.from(activeModelsRef.current.entries())) {
      if (!openSet.has(path)) {
        activeModelsRef.current.delete(path);
        cachedModelsRef.current.set(path, model);
      }
    }

    while (cachedModelsRef.current.size > MAX_CACHED_MODELS) {
      const firstKey = cachedModelsRef.current.keys().next().value as string;
      cachedModelsRef.current.get(firstKey)?.dispose();
      cachedModelsRef.current.delete(firstKey);
    }
  }, [openPaths]);

  useEffect(() => {
    return () => {
      for (const model of Array.from(activeModelsRef.current.values())) {
        model.dispose();
      }
      activeModelsRef.current.clear();
      for (const model of Array.from(cachedModelsRef.current.values())) {
        model.dispose();
      }
      cachedModelsRef.current.clear();
    };
  }, []);

  return (
    <Editor
      height="100%"
      language={getLanguage(filePath)}
      defaultValue={value}
      theme={toMonacoTheme(theme)}
      options={{
        readOnly: mode === "readonly",
        minimap: { enabled: false },
        fontSize: 13,
        lineNumbers: "on",
        wordWrap: "on",
        padding: { top: 8 },
        automaticLayout: true,
        scrollBeyondLastLine: false,
      }}
      onMount={handleMount}
    />
  );
}
