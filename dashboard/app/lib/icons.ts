export function getFileIcon(name: string): string {
  const ext = name.split(".").pop()?.toLowerCase() ?? "";
  const map: Record<string, string> = {
    ts: "lucide:file-type",
    tsx: "lucide:file-type",
    js: "lucide:file-code-2",
    jsx: "lucide:file-code-2",
    json: "lucide:braces",
    css: "lucide:paintbrush",
    html: "lucide:file-code",
    md: "lucide:file-text",
    toml: "lucide:settings-2",
    rs: "lucide:file-code-2",
    yml: "lucide:file-json",
    yaml: "lucide:file-json",
  };
  return map[ext] ?? "lucide:file";
}

export function getFolderIcon(): string {
  return "lucide:folder";
}
