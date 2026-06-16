import {
  Disclosure,
  DisclosureTrigger,
  DisclosureHeading,
  DisclosureContent,
  DisclosureBody,
  DisclosureIndicator,
  DisclosureGroup,
} from "@heroui/react";
import { Icon } from "@iconify/react";

interface FileNode {
  name: string;
  path: string;
  type: "file" | "directory";
  children?: FileNode[];
}

function buildTree(paths: string[]): FileNode[] {
  const root: FileNode[] = [];

  for (const filePath of paths) {
    const parts = filePath.startsWith("/") ? filePath.slice(1).split("/") : filePath.split("/");
    let current = root;

    for (let i = 0; i < parts.length; i++) {
      const name = parts[i];
      const isFile = i === parts.length - 1;
      const fullPath = "/" + parts.slice(0, i + 1).join("/");

      let existing = current.find((n) => n.name === name);
      if (!existing) {
        existing = {
          name,
          path: fullPath,
          type: isFile ? "file" : "directory",
          children: isFile ? undefined : [],
        };
        current.push(existing);
      }
      if (existing.children) {
        current = existing.children;
      }
    }
  }

  return sortTree(root);
}

function sortTree(nodes: FileNode[]): FileNode[] {
  return nodes.sort((a, b) => {
    if (a.type !== b.type) return a.type === "directory" ? -1 : 1;
    return a.name.localeCompare(b.name);
  }).map((node) => ({
    ...node,
    children: node.children ? sortTree(node.children) : undefined,
  }));
}

const fileIcons: Record<string, string> = {
  ts: "lucide:file-type",
  tsx: "lucide:file-type",
  js: "lucide:file-code-2",
  jsx: "lucide:file-code-2",
  json: "lucide:braces",
  css: "lucide:paintbrush",
  html: "lucide:file-code",
  md: "lucide:file-text",
};

function getFileIcon(name: string): string {
  const ext = name.split(".").pop()?.toLowerCase() ?? "";
  return fileIcons[ext] ?? "lucide:file";
}

export function FileTree({
  files,
  selectedPath,
  onSelect,
}: {
  files: Record<string, string>;
  selectedPath?: string;
  onSelect: (path: string) => void;
}) {
  const tree = buildTree(Object.keys(files));

  return (
    <div className="text-sm">
      <FileTreeNodeList
        nodes={tree}
        selectedPath={selectedPath}
        onSelect={onSelect}
      />
    </div>
  );
}

function FileTreeNodeList({
  nodes,
  selectedPath,
  onSelect,
}: {
  nodes: FileNode[];
  selectedPath?: string;
  onSelect: (path: string) => void;
}) {
  return (
    <DisclosureGroup>
      {nodes.map((node) =>
        node.type === "directory" && node.children ? (
          <Disclosure key={node.path} defaultExpanded>
            <DisclosureTrigger className="flex items-center gap-2 w-full px-2 py-1 rounded-md text-left text-sm hover:bg-surface-secondary transition-colors">
              <DisclosureIndicator>
                <Icon icon="lucide:chevron-right" className="w-3.5 h-3.5 text-muted" />
              </DisclosureIndicator>
              <Icon icon="lucide:folder" className="w-3.5 h-3.5 text-muted" />
              <DisclosureHeading className="text-sm font-normal truncate">{node.name}</DisclosureHeading>
            </DisclosureTrigger>
            <DisclosureContent>
              <DisclosureBody className="pl-4">
                <FileTreeNodeList
                  nodes={node.children}
                  selectedPath={selectedPath}
                  onSelect={onSelect}
                />
              </DisclosureBody>
            </DisclosureContent>
          </Disclosure>
        ) : (
          <button
            key={node.path}
            className={`flex items-center gap-2 w-full px-2 py-1 rounded-md text-left text-sm hover:bg-surface-secondary transition-colors ${
              selectedPath === node.path ? "bg-accent-soft text-accent" : ""
            }`}
            onClick={() => onSelect(node.path)}
          >
            <Icon icon={getFileIcon(node.name)} className="w-3.5 h-3.5 text-muted" />
            <span className="truncate">{node.name}</span>
          </button>
        ),
      )}
    </DisclosureGroup>
  );
}
