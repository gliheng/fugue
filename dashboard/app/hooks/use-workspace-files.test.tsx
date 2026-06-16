import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { useMemo } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { api } from "../lib/api";
import { useWorkspaceFiles } from "./use-workspace-files";
import type { Workspace } from "../lib/types";

const workspaceResponse: Workspace = {
  id: "ws1",
  name: "test",
  framework: "worker",
  file_count: 1,
  created_at: new Date().toISOString(),
  updated_at: new Date().toISOString(),
};

const updateWorkspaceSpy = vi.spyOn(api, "updateWorkspace");

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });
  return {
    queryClient,
    Wrapper({ children }: { children: React.ReactNode }) {
      return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
    },
  };
}

describe("useWorkspaceFiles", () => {
  beforeEach(() => {
    updateWorkspaceSpy.mockReset();
    updateWorkspaceSpy.mockResolvedValue(workspaceResponse);
  });

  it("auto-saves a dirty file after the configured delay", async () => {
    const { Wrapper, queryClient } = createWrapper();
    const { result, unmount } = renderHook(
      () => {
        const initialFiles = useMemo(() => ({ "/a.ts": "original" }), []);
        return useWorkspaceFiles({
          workspaceId: "ws1",
          initialFiles,
          saveStrategy: "auto",
          autoSaveDelay: 50,
        });
      },
      { wrapper: Wrapper },
    );

    act(() => {
      result.current.updateContent("/a.ts", "edited");
    });

    expect(result.current.files["/a.ts"].status).toBe("dirty");
    expect(api.updateWorkspace).not.toHaveBeenCalled();

    await waitFor(() => expect(api.updateWorkspace).toHaveBeenCalledWith("ws1", { files: { "/a.ts": "edited" } }));
    expect(result.current.files["/a.ts"].status).toBe("clean");

    unmount();
    queryClient.clear();
  });

  it("does not auto-save when saveStrategy is manual", async () => {
    const { Wrapper, queryClient } = createWrapper();
    const { result, unmount } = renderHook(
      () => {
        const initialFiles = useMemo(() => ({ "/a.ts": "original" }), []);
        return useWorkspaceFiles({
          workspaceId: "ws1",
          initialFiles,
          saveStrategy: "manual",
        });
      },
      { wrapper: Wrapper },
    );

    act(() => {
      result.current.updateContent("/a.ts", "edited");
    });

    await new Promise((resolve) => setTimeout(resolve, 100));
    expect(api.updateWorkspace).not.toHaveBeenCalled();

    await act(async () => {
      await result.current.saveFile("/a.ts");
    });

    expect(api.updateWorkspace).toHaveBeenCalledWith("ws1", { files: { "/a.ts": "edited" } });

    unmount();
    queryClient.clear();
  });

  it("keeps a file dirty when the user edits while a save is in flight", async () => {
    const { Wrapper, queryClient } = createWrapper();
    let resolveSave: ((value: Workspace) => void) | undefined;
    updateWorkspaceSpy.mockImplementation(
      () => new Promise<Workspace>((resolve) => {
        resolveSave = resolve;
      }),
    );

    const { result, unmount } = renderHook(
      () => {
        const initialFiles = useMemo(() => ({ "/a.ts": "v0" }), []);
        return useWorkspaceFiles({
          workspaceId: "ws1",
          initialFiles,
          saveStrategy: "auto",
          autoSaveDelay: 50,
        });
      },
      { wrapper: Wrapper },
    );

    act(() => {
      result.current.updateContent("/a.ts", "v1");
    });

    await waitFor(() => expect(result.current.files["/a.ts"].status).toBe("saving"));

    act(() => {
      result.current.updateContent("/a.ts", "v2");
    });

    // While the save is in flight the file stays in the saving state.
    expect(result.current.files["/a.ts"].status).toBe("saving");
    expect(result.current.files["/a.ts"].currentContent).toBe("v2");

    await act(async () => {
      resolveSave?.(workspaceResponse);
    });

    // Once the save completes, the concurrent edit keeps the file dirty.
    await waitFor(() => expect(result.current.files["/a.ts"].status).toBe("dirty"));
    expect(result.current.files["/a.ts"].serverContent).toBe("v1");
    expect(result.current.files["/a.ts"].currentContent).toBe("v2");

    unmount();
    queryClient.clear();
  });

  it("saves all dirty files with saveAll", async () => {
    const { Wrapper, queryClient } = createWrapper();
    const { result, unmount } = renderHook(
      () => {
        const initialFiles = useMemo(() => ({ "/a.ts": "a", "/b.ts": "b" }), []);
        return useWorkspaceFiles({
          workspaceId: "ws1",
          initialFiles,
          saveStrategy: "manual",
        });
      },
      { wrapper: Wrapper },
    );

    act(() => {
      result.current.updateContent("/a.ts", "a2");
      result.current.updateContent("/b.ts", "b2");
    });

    await act(async () => {
      await result.current.saveAll();
    });

    expect(api.updateWorkspace).toHaveBeenCalledWith("ws1", {
      files: { "/a.ts": "a2", "/b.ts": "b2" },
    });

    unmount();
    queryClient.clear();
  });
});
