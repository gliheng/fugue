import { useCallback, useEffect, useMemo, useReducer, useRef } from "react";
import { useMutation } from "@tanstack/react-query";
import { api } from "../lib/api";
import {
  fileStateReducer,
  getDirtyFiles,
  type FileState,
  type WorkspaceFilesState,
} from "../lib/file-state";

export type SaveStrategy = "manual" | "auto";

export interface UseWorkspaceFilesOptions {
  workspaceId: string;
  initialFiles: Record<string, string>;
  saveStrategy?: SaveStrategy;
  autoSaveDelay?: number;
}

export interface UseWorkspaceFilesResult {
  files: Record<string, FileState>;
  openPaths: string[];
  activePath: string | null;
  dirtyFiles: FileState[];
  openFile: (path: string) => void;
  setActiveFile: (path: string | null) => void;
  closeFile: (path: string) => void;
  updateContent: (path: string, content: string) => void;
  saveFile: (path: string) => Promise<void>;
  saveAll: () => Promise<void>;
}

function normalizePath(path: string): string {
  return path.startsWith("/") ? path : `/${path}`;
}

export function useWorkspaceFiles({
  workspaceId,
  initialFiles,
  saveStrategy = "auto",
  autoSaveDelay = 1000,
}: UseWorkspaceFilesOptions): UseWorkspaceFilesResult {
  const [state, dispatch] = useReducer(fileStateReducer, {
    files: {},
    openPaths: [],
    activePath: null,
  });
  const stateRef = useRef(state);
  stateRef.current = state;

  useEffect(() => {
    dispatch({ type: "INIT", payload: { files: initialFiles } });
  }, [initialFiles]);

  const saveMutation = useMutation({
    mutationFn: (files: Record<string, string>) => api.updateWorkspace(workspaceId, { files }),
  });

  const autoSaveTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const performSave = useCallback(
    async (paths: string[]) => {
      const currentState = stateRef.current;
      const filesToSave: Record<string, string> = {};
      for (const path of paths) {
        const normalized = normalizePath(path);
        const file = currentState.files[normalized];
        if (file) {
          filesToSave[normalized] = file.currentContent;
        }
      }

      dispatch({ type: "MARK_SAVING", payload: { paths } });

      try {
        await saveMutation.mutateAsync(filesToSave);
        dispatch({ type: "MARK_SAVED", payload: { files: filesToSave } });
      } catch (e) {
        const message = e instanceof Error ? e.message : "Save failed";
        for (const path of paths) {
          dispatch({ type: "MARK_ERROR", payload: { path, error: message } });
        }
        throw e;
      }
    },
    [saveMutation],
  );

  const updateContent = useCallback(
    (path: string, content: string) => {
      const normalized = normalizePath(path);
      dispatch({ type: "UPDATE_CONTENT", payload: { path: normalized, content } });

      if (saveStrategy === "auto") {
        if (autoSaveTimeoutRef.current) clearTimeout(autoSaveTimeoutRef.current);
        autoSaveTimeoutRef.current = setTimeout(() => {
          const file = stateRef.current.files[normalized];
          if (file && file.currentContent !== file.serverContent) {
            performSave([normalized]).catch(() => {});
          }
        }, autoSaveDelay);
      }
    },
    [saveStrategy, autoSaveDelay, performSave],
  );

  useEffect(() => {
    return () => {
      if (autoSaveTimeoutRef.current) clearTimeout(autoSaveTimeoutRef.current);
    };
  }, []);

  const openFile = useCallback((path: string) => {
    dispatch({ type: "OPEN", payload: { path } });
  }, []);

  const setActiveFile = useCallback((path: string | null) => {
    dispatch({ type: "SET_ACTIVE", payload: { path } });
  }, []);

  const closeFile = useCallback((path: string) => {
    dispatch({ type: "CLOSE", payload: { path } });
  }, []);

  const saveFile = useCallback(
    async (path: string) => {
      const normalized = normalizePath(path);
      const file = stateRef.current.files[normalized];
      if (!file || file.currentContent === file.serverContent) return;
      await performSave([normalized]);
    },
    [performSave],
  );

  const saveAll = useCallback(async () => {
    const changed = getDirtyFiles(stateRef.current);
    if (changed.length === 0) return;
    await performSave(changed.map((f) => f.path));
  }, [performSave]);

  const dirtyFiles = useMemo(() => getDirtyFiles(state), [state]);

  return {
    files: state.files,
    openPaths: state.openPaths,
    activePath: state.activePath,
    dirtyFiles,
    openFile,
    setActiveFile,
    closeFile,
    updateContent,
    saveFile,
    saveAll,
  };
}
