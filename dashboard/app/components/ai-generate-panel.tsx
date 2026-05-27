import { useCallback, useRef, useState } from "react";
import { Button, Spinner } from "@heroui/react";
import { Icon } from "@iconify/react";
import { api } from "../lib/api";

interface AiGeneratePanelProps {
  framework: string;
  workspaceId: string;
  onGenerated: (files: Record<string, string>) => void;
  onClose: () => void;
}

export function AiGeneratePanel({
  framework,
  workspaceId,
  onGenerated,
  onClose,
}: AiGeneratePanelProps) {
  const [prompt, setPrompt] = useState("");
  const [generating, setGenerating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [streamTokens, setStreamTokens] = useState<string[]>([]);
  const [fileCount, setFileCount] = useState(0);
  const tokensEndRef = useRef<HTMLDivElement>(null);

  const handleGenerate = useCallback(async () => {
    if (!prompt.trim()) return;

    setGenerating(true);
    setError(null);
    setStreamTokens([]);
    setFileCount(0);

    try {
      const stream = api.generateAI(prompt.trim(), workspaceId);

      for await (const event of stream) {
        if (event.event_type === "token") {
          setStreamTokens((prev) => {
            const updated = [...prev, event.data.text ?? ""];
            // Auto-scroll to bottom
            setTimeout(() => tokensEndRef.current?.scrollIntoView({ behavior: "smooth" }), 50);
            return updated;
          });
        } else if (event.event_type === "file") {
          setFileCount((c) => c + 1);
        } else if (event.event_type === "done") {
          if (event.data.files) {
            onGenerated(event.data.files);
          }
        } else if (event.event_type === "error") {
          setError(event.data.error ?? "Unknown error");
        }
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : "Generation failed");
    } finally {
      setGenerating(false);
    }
  }, [prompt, workspaceId, onGenerated]);

  const frameworkLabel =
    framework === "react-router"
      ? "React Router"
      : framework === "nuxtjs"
        ? "Nuxt.js"
        : "Worker";

  const tokensText = streamTokens.join("");

  return (
    <div className="flex flex-col h-full border-l border-border bg-surface-secondary">
      <div className="flex items-center justify-between px-4 py-3 border-b border-border">
        <div className="flex items-center gap-2">
          <Icon icon="lucide:sparkles" className="w-4 h-4 text-accent" />
          <span className="text-sm font-semibold">AI Generate</span>
        </div>
        <button
          className="text-muted hover:text-foreground transition-colors"
          onClick={onClose}
        >
          <Icon icon="lucide:x" className="w-4 h-4" />
        </button>
      </div>

      <div className="flex-1 flex flex-col overflow-hidden">
        <div className="p-4 space-y-3">
          <div className="flex items-center gap-2">
            <span className="text-xs text-muted">Framework:</span>
            <span className="text-xs px-2 py-0.5 rounded bg-surface-tertiary font-mono">
              {frameworkLabel}
            </span>
          </div>

          <textarea
            className="w-full h-28 p-3 text-sm rounded-lg border border-border bg-surface-primary text-foreground resize-none focus:outline-none focus:border-accent placeholder:text-muted"
            placeholder="Describe the app you want to build...&#10;&#10;e.g. Create a personal blog with article list and detail pages, using Markdown rendering"
            value={prompt}
            onChange={(e) => setPrompt(e.target.value)}
            disabled={generating}
          />

          <Button
            onPress={handleGenerate}
            isDisabled={!prompt.trim() || generating}
            className="w-full"
          >
            {generating ? (
              <>
                <Spinner color="current" size="sm" />
                Generating...
              </>
            ) : (
              <>
                <Icon icon="lucide:sparkles" className="w-4 h-4" />
                Generate
              </>
            )}
          </Button>
        </div>

        {error && (
          <div className="mx-4 mb-3 p-3 rounded-lg bg-danger-soft text-sm text-danger">
            <Icon icon="lucide:alert-circle" className="w-4 h-4 inline mr-1" />
            {error}
          </div>
        )}

        {(generating || streamTokens.length > 0) && (
          <div className="flex-1 mx-4 mb-4 overflow-hidden flex flex-col">
            <div className="flex items-center justify-between mb-2">
              <span className="text-xs text-muted font-medium">Output</span>
              {fileCount > 0 && (
                <span className="text-xs text-accent">
                  {fileCount} file{fileCount !== 1 ? "s" : ""} generated
                </span>
              )}
            </div>
            <div className="flex-1 overflow-auto rounded-lg bg-surface-primary border border-border p-3">
              <pre className="text-xs font-mono text-foreground whitespace-pre-wrap break-all leading-relaxed">
                {tokensText}
                {generating && (
                  <span className="inline-block w-1.5 h-3 bg-accent animate-pulse ml-0.5" />
                )}
              </pre>
              <div ref={tokensEndRef} />
            </div>
          </div>
        )}

        {!generating && streamTokens.length === 0 && !error && (
          <div className="flex-1 flex items-center justify-center px-4">
            <div className="text-center text-muted">
              <Icon icon="lucide:wand-2" className="w-8 h-8 mx-auto mb-2 opacity-40" />
              <p className="text-xs">
                Describe your app and the AI will generate the complete project
              </p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
