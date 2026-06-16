import "@testing-library/jest-dom";
import { vi } from "vitest";

// Mock @iconify/react to avoid network icon fetches in jsdom
vi.mock("@iconify/react", () => ({
  Icon: ({ icon, className }: { icon: string; className?: string }) => (
    <span data-testid="icon" data-icon={icon} className={className} />
  ),
}));

// Mock @monaco-editor/react's Editor component
vi.mock("@monaco-editor/react", () => ({
  Editor: ({ defaultValue, value, onChange, onMount }: {
    defaultValue?: string;
    value?: string;
    onChange?: (value: string) => void;
    onMount?: (editor: unknown, monaco: unknown) => void;
  }) => {
    const initialValue = value ?? defaultValue ?? "";
    // Simulate calling onMount with a minimal editor stub
    if (onMount) {
      onMount(
        {
          getValue: () => initialValue,
          getModel: () => null,
          setModel: () => {},
          onDidChangeModelContent: (cb: () => void) => ({ dispose: () => {} }),
        },
        {
          editor: {
            getModel: () => null,
            createModel: (v: string) => ({ getValue: () => v, setValue: () => {}, dispose: () => {} }),
          },
          Uri: { file: (path: string) => ({ path }) },
        },
      );
    }
    return (
      <textarea
        data-testid="monaco-editor"
        defaultValue={initialValue}
        onChange={(e) => onChange?.(e.target.value)}
        readOnly={!onChange}
      />
    );
  },
}));

// Provide a basic matchMedia stub
Object.defineProperty(window, "matchMedia", {
  writable: true,
  value: vi.fn().mockImplementation((query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
});
