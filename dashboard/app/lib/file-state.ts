export type FileStatus = "clean" | "dirty" | "saving" | "error";

export interface FileState {
  path: string;
  serverContent: string;
  currentContent: string;
  status: FileStatus;
  error?: string;
}

export interface WorkspaceFilesState {
  files: Record<string, FileState>;
  openPaths: string[];
  activePath: string | null;
}

export type WorkspaceFilesAction =
  | { type: "INIT"; payload: { files: Record<string, string>; activePath?: string | null } }
  | { type: "OPEN"; payload: { path: string } }
  | { type: "SET_ACTIVE"; payload: { path: string | null } }
  | { type: "CLOSE"; payload: { path: string } }
  | { type: "UPDATE_CONTENT"; payload: { path: string; content: string } }
  | { type: "MARK_SAVING"; payload: { paths: string[] } }
  | { type: "MARK_SAVED"; payload: { files: Record<string, string> } }
  | { type: "MARK_ERROR"; payload: { path: string; error: string } };

export function createFileState(path: string, content: string): FileState {
  return {
    path,
    serverContent: content,
    currentContent: content,
    status: "clean",
  };
}

function normalizePath(path: string): string {
  return path.startsWith("/") ? path : `/${path}`;
}

function computeStatus(state: FileState): FileStatus {
  if (state.status === "saving") return "saving";
  if (state.status === "error") return state.currentContent === state.serverContent ? "error" : "dirty";
  return state.currentContent === state.serverContent ? "clean" : "dirty";
}

export function fileStateReducer(
  state: WorkspaceFilesState,
  action: WorkspaceFilesAction,
): WorkspaceFilesState {
  switch (action.type) {
    case "INIT": {
      const files: Record<string, FileState> = {};
      for (const [path, content] of Object.entries(action.payload.files)) {
        const normalized = normalizePath(path);
        const existing = state.files[normalized];
        if (existing) {
          // Merge server content; preserve user's currentContent and status.
          const becameClean = existing.currentContent === content;
          files[normalized] = {
            ...existing,
            serverContent: content,
            status: becameClean ? "clean" : existing.status === "saving" ? existing.status : "dirty",
            error: becameClean ? undefined : existing.error,
          };
        } else {
          files[normalized] = createFileState(normalized, content);
        }
      }

      // Drop open paths and active path that no longer exist on the server.
      const openPaths = state.openPaths.filter((p) => files[normalizePath(p)] != null);
      let activePath = state.activePath;
      if (activePath && !files[activePath]) {
        activePath = openPaths[openPaths.length - 1] ?? null;
      }

      return {
        ...state,
        files,
        openPaths,
        activePath,
      };
    }

    case "OPEN": {
      const path = normalizePath(action.payload.path);
      if (!state.files[path]) return state;
      const openPaths = state.openPaths.includes(path) ? state.openPaths : [...state.openPaths, path];
      return { ...state, openPaths, activePath: path };
    }

    case "SET_ACTIVE": {
      const path = action.payload.path ? normalizePath(action.payload.path) : null;
      return { ...state, activePath: path };
    }

    case "CLOSE": {
      const path = normalizePath(action.payload.path);
      const openPaths = state.openPaths.filter((p) => p !== path);
      let activePath = state.activePath;
      if (state.activePath === path) {
        activePath = openPaths[openPaths.length - 1] ?? null;
      }
      return { ...state, openPaths, activePath };
    }

    case "UPDATE_CONTENT": {
      const path = normalizePath(action.payload.path);
      const file = state.files[path];
      if (!file || file.currentContent === action.payload.content) return state;
      const updated: FileState = {
        ...file,
        currentContent: action.payload.content,
        status: computeStatus({ ...file, currentContent: action.payload.content }),
      };
      return { ...state, files: { ...state.files, [path]: updated } };
    }

    case "MARK_SAVING": {
      const nextFiles = { ...state.files };
      for (const path of action.payload.paths) {
        const normalized = normalizePath(path);
        const file = nextFiles[normalized];
        if (file) {
          nextFiles[normalized] = { ...file, status: "saving", error: undefined };
        }
      }
      return { ...state, files: nextFiles };
    }

    case "MARK_SAVED": {
      const nextFiles = { ...state.files };
      for (const [path, content] of Object.entries(action.payload.files)) {
        const normalized = normalizePath(path);
        const file = nextFiles[normalized];
        if (file) {
          nextFiles[normalized] = {
            ...file,
            serverContent: content,
            status: content === file.currentContent ? "clean" : "dirty",
            error: undefined,
          };
        }
      }
      return { ...state, files: nextFiles };
    }

    case "MARK_ERROR": {
      const path = normalizePath(action.payload.path);
      const file = state.files[path];
      if (!file) return state;
      return {
        ...state,
        files: {
          ...state.files,
          [path]: { ...file, status: "error", error: action.payload.error },
        },
      };
    }

    default:
      return state;
  }
}

export function getDirtyFiles(state: WorkspaceFilesState): FileState[] {
  return Object.values(state.files).filter((f) => f.status === "dirty" || f.status === "error");
}
