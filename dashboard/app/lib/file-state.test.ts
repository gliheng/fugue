import { describe, it, expect } from "vitest";
import { fileStateReducer, createFileState, getDirtyFiles } from "./file-state";

describe("fileStateReducer", () => {
  it("INIT creates new file states from server files", () => {
    const state = fileStateReducer(
      { files: {}, openPaths: [], activePath: null },
      { type: "INIT", payload: { files: { "/a.ts": "a", "/b.ts": "b" } } },
    );

    expect(state.files["/a.ts"]).toEqual(createFileState("/a.ts", "a"));
    expect(state.files["/b.ts"]).toEqual(createFileState("/b.ts", "b"));
  });

  it("INIT merges server content without overwriting unsaved edits", () => {
    const initial = fileStateReducer(
      { files: {}, openPaths: [], activePath: null },
      { type: "INIT", payload: { files: { "/a.ts": "original" } } },
    );

    const edited = fileStateReducer(
      initial,
      { type: "UPDATE_CONTENT", payload: { path: "/a.ts", content: "edited" } },
    );

    const refetched = fileStateReducer(
      edited,
      { type: "INIT", payload: { files: { "/a.ts": "server-changed" } } },
    );

    expect(refetched.files["/a.ts"].currentContent).toBe("edited");
    expect(refetched.files["/a.ts"].serverContent).toBe("server-changed");
    expect(refetched.files["/a.ts"].status).toBe("dirty");
  });

  it("INIT marks a file clean when server matches current content", () => {
    const initial = fileStateReducer(
      { files: {}, openPaths: [], activePath: null },
      { type: "INIT", payload: { files: { "/a.ts": "original" } } },
    );

    const saved = fileStateReducer(
      initial,
      { type: "MARK_SAVED", payload: { files: { "/a.ts": "original" } } },
    );

    const refetched = fileStateReducer(
      saved,
      { type: "INIT", payload: { files: { "/a.ts": "original" } } },
    );

    expect(refetched.files["/a.ts"].status).toBe("clean");
  });

  it("MARK_SAVED clears error and preserves concurrent edits", () => {
    const initial = fileStateReducer(
      { files: {}, openPaths: [], activePath: null },
      { type: "INIT", payload: { files: { "/a.ts": "v1" } } },
    );

    const edited = fileStateReducer(
      initial,
      { type: "UPDATE_CONTENT", payload: { path: "/a.ts", content: "v2" } },
    );

    const saving = fileStateReducer(
      edited,
      { type: "MARK_SAVING", payload: { paths: ["/a.ts"] } },
    );

    // User types more while save is in flight
    const editedAgain = fileStateReducer(
      saving,
      { type: "UPDATE_CONTENT", payload: { path: "/a.ts", content: "v3" } },
    );

    const saved = fileStateReducer(
      editedAgain,
      { type: "MARK_SAVED", payload: { files: { "/a.ts": "v2" } } },
    );

    expect(saved.files["/a.ts"].currentContent).toBe("v3");
    expect(saved.files["/a.ts"].serverContent).toBe("v2");
    expect(saved.files["/a.ts"].status).toBe("dirty");
  });

  it("MARK_ERROR preserves status and stores message", () => {
    const initial = fileStateReducer(
      { files: {}, openPaths: [], activePath: null },
      { type: "INIT", payload: { files: { "/a.ts": "a" } } },
    );

    const errored = fileStateReducer(
      initial,
      { type: "MARK_ERROR", payload: { path: "/a.ts", error: "Network error" } },
    );

    expect(errored.files["/a.ts"].status).toBe("error");
    expect(errored.files["/a.ts"].error).toBe("Network error");
  });

  it("CLOSE removes path from openPaths and updates activePath", () => {
    const state = fileStateReducer(
      { files: {}, openPaths: ["/a.ts", "/b.ts"], activePath: "/b.ts" },
      { type: "CLOSE", payload: { path: "/b.ts" } },
    );

    expect(state.openPaths).toEqual(["/a.ts"]);
    expect(state.activePath).toBe("/a.ts");
  });

  it("OPEN adds path to openPaths and makes it active", () => {
    const state = fileStateReducer(
      { files: { "/a.ts": createFileState("/a.ts", "a") }, openPaths: [], activePath: null },
      { type: "OPEN", payload: { path: "/a.ts" } },
    );

    expect(state.openPaths).toEqual(["/a.ts"]);
    expect(state.activePath).toBe("/a.ts");
  });

  it("INIT filters out openPaths and activePath for files removed on the server", () => {
    const state = fileStateReducer(
      {
        files: {
          "/keep.ts": createFileState("/keep.ts", "keep"),
          "/remove.ts": createFileState("/remove.ts", "remove"),
        },
        openPaths: ["/keep.ts", "/remove.ts"],
        activePath: "/remove.ts",
      },
      { type: "INIT", payload: { files: { "/keep.ts": "keep" } } },
    );

    expect(state.openPaths).toEqual(["/keep.ts"]);
    expect(state.activePath).toBe("/keep.ts");
  });
});

describe("getDirtyFiles", () => {
  it("returns dirty and error files", () => {
    const state = {
      files: {
        "/clean.ts": createFileState("/clean.ts", "clean"),
        "/dirty.ts": { ...createFileState("/dirty.ts", "x"), currentContent: "y", status: "dirty" as const },
        "/error.ts": { ...createFileState("/error.ts", "x"), status: "error" as const, error: "boom" },
      },
      openPaths: [],
      activePath: null,
    };

    const dirty = getDirtyFiles(state);
    expect(dirty.map((f) => f.path)).toEqual(["/dirty.ts", "/error.ts"]);
  });
});
