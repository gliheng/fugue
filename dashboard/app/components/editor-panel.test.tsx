import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { EditorPanel } from "./editor-panel";
import type { FileState } from "../lib/file-state";

function makeFile(path: string, overrides: Partial<FileState> = {}): FileState {
  const content = overrides.currentContent ?? overrides.serverContent ?? `content of ${path}`;
  return {
    path,
    serverContent: overrides.serverContent ?? content,
    currentContent: content,
    status: "clean",
    ...overrides,
  };
}

function renderPanel(props: Partial<Parameters<typeof EditorPanel>[0]> = {}) {
  const files: Record<string, FileState> = {
    "/package.json": makeFile("/package.json", { currentContent: '{"name":"app"}' }),
    "/src/index.ts": makeFile("/src/index.ts", { currentContent: "console.log(1)" }),
  };

  const defaultProps = {
    files,
    activePath: "/package.json",
    openPaths: ["/package.json", "/src/index.ts"],
    onOpen: vi.fn(),
    onClose: vi.fn(),
    onChange: vi.fn(),
    onSave: vi.fn(),
    onSaveAll: vi.fn(),
    mode: "edit" as const,
    saveStrategy: "auto" as const,
  };

  return render(<EditorPanel {...defaultProps} {...props} />);
}

describe("EditorPanel", () => {
  it("renders tabs for open files and highlights the active tab", () => {
    renderPanel();
    expect(screen.getByText("package.json")).toBeInTheDocument();
    expect(screen.getByText("index.ts")).toBeInTheDocument();
    expect(screen.getByTestId("monaco-editor")).toHaveValue('{"name":"app"}');
  });

  it("calls onOpen when a tab is clicked", async () => {
    const onOpen = vi.fn();
    renderPanel({ onOpen });
    await userEvent.click(screen.getByText("index.ts"));
    expect(onOpen).toHaveBeenCalledWith("/src/index.ts");
  });

  it("calls onClose when close button is clicked for a clean tab", async () => {
    const onClose = vi.fn();
    renderPanel({ onClose });
    // The close button has an aria-label; user-event clicking it should not bubble to the tab button
    const closeButton = screen.getByLabelText(/Close index\.ts/i);
    await userEvent.click(closeButton);
    expect(onClose).toHaveBeenCalledWith("/src/index.ts");
  });

  it("shows a confirmation modal before closing a dirty tab", async () => {
    const onClose = vi.fn();
    const files: Record<string, FileState> = {
      "/package.json": makeFile("/package.json", { currentContent: '{"name":"changed"}', serverContent: '{"name":"app"}', status: "dirty" }),
    };
    renderPanel({ files, openPaths: ["/package.json"], activePath: "/package.json", onClose });

    await userEvent.click(screen.getByLabelText(/Close package\.json/i));
    expect(screen.getByRole("heading", { name: /unsaved changes/i })).toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: /close without saving/i }));
    expect(onClose).toHaveBeenCalledWith("/package.json");
  });

  it("calls onSave when Ctrl+S is pressed", async () => {
    const onSave = vi.fn();
    renderPanel({ onSave });
    fireEvent.keyDown(window, { key: "s", ctrlKey: true });
    await waitFor(() => expect(onSave).toHaveBeenCalledWith("/package.json"));
  });

  it("calls onClose when Ctrl+W is pressed", async () => {
    const onClose = vi.fn();
    renderPanel({ onClose });
    fireEvent.keyDown(window, { key: "w", ctrlKey: true });
    await waitFor(() => expect(onClose).toHaveBeenCalledWith("/package.json"));
  });

  it("displays a save error and calls onSave when retry is clicked", async () => {
    const onSave = vi.fn();
    const files: Record<string, FileState> = {
      "/package.json": makeFile("/package.json", { status: "error", error: "Network error" }),
    };
    renderPanel({ files, openPaths: ["/package.json"], activePath: "/package.json", onSave });

    expect(screen.getByText(/network error/i)).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: /retry/i }));
    expect(onSave).toHaveBeenCalledWith("/package.json");
  });

  it("shows dirty indicator and save-all button when there are dirty files", () => {
    const files: Record<string, FileState> = {
      "/package.json": makeFile("/package.json", { currentContent: "changed", serverContent: "original", status: "dirty" }),
      "/src/index.ts": makeFile("/src/index.ts"),
    };
    renderPanel({ files });
    expect(screen.getByText(/save all/i)).toBeInTheDocument();
  });

  it("does not show save controls in readonly mode", () => {
    renderPanel({ mode: "readonly" });
    expect(screen.queryByRole("button", { name: /save/i })).not.toBeInTheDocument();
  });
});
