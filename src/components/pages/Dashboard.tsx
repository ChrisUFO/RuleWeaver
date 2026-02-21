import { useEffect, useState } from "react";
import {
  Plus,
  RefreshCw,
  FileText,
  Globe,
  FolderOpen,
  Clock,
  CheckCircle,
  XCircle,
  AlertTriangle,
  History,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { useRulesStore } from "@/stores/rulesStore";
import { useToast } from "@/components/ui/toast";
import { api } from "@/lib/tauri";
import { SyncPreviewDialog } from "@/components/sync/SyncPreviewDialog";
import { SyncProgress } from "@/components/sync/SyncProgress";
import { SyncResultsDialog } from "@/components/sync/SyncResultsDialog";
import type { SyncResult, CreateRuleInput, AdapterType, SyncHistoryEntry } from "@/types/rule";

interface Template {
  name: string;
  icon: string;
  content: string;
  adapters: AdapterType[];
}

const TEMPLATES: Template[] = [
  {
    name: "TypeScript Best Practices",
    icon: "TS",
    content: `# TypeScript Best Practices

## Code Style
- Use strict TypeScript configuration
- Prefer interfaces over type aliases for object shapes
- Use const assertions for literal types
- Avoid any; use unknown when type is uncertain

## Naming Conventions
- Use PascalCase for types, interfaces, and classes
- Use camelCase for variables and functions
- Use SCREAMING_SNAKE_CASE for constants

## Error Handling
- Always handle promise rejections
- Use typed errors with error codes
- Log errors with context for debugging`,
    adapters: ["antigravity", "gemini", "opencode"] as AdapterType[],
  },
  {
    name: "React Components",
    icon: "Re",
    content: `# React Components

## Component Structure
- Use functional components with hooks
- One component per file
- Keep components small and focused
- Extract reusable logic into custom hooks

## Props
- Define prop types with TypeScript interfaces
- Use optional props with sensible defaults
- Destructure props in function signature

## State Management
- Use useState for local component state
- Lift state up when needed by siblings
- Consider useReducer for complex state logic

## Performance
- Memoize expensive computations with useMemo
- Use React.memo for pure components
- Avoid inline function definitions in render`,
    adapters: ["antigravity", "cline", "claude-code"] as AdapterType[],
  },
  {
    name: "Python Standards",
    icon: "Py",
    content: `# Python Standards

## Code Style
- Follow PEP 8 conventions
- Use type hints for function signatures
- Maximum line length of 88 characters (Black default)
- Use f-strings for string formatting

## Documentation
- Write docstrings for all public functions
- Use Google style docstrings
- Include examples in docstrings

## Error Handling
- Use specific exception types
- Never catch exceptions silently
- Use context managers for resource handling

## Testing
- Write unit tests with pytest
- Aim for high test coverage
- Use fixtures for test setup`,
    adapters: ["gemini", "opencode", "codex"] as AdapterType[],
  },
  {
    name: "Git Commit Rules",
    icon: "Gi",
    content: `# Git Commit Rules

## Commit Messages
- Use conventional commit format
- Keep subject line under 72 characters
- Use imperative mood in subject line
- Separate subject from body with blank line

## Commit Types
- feat: New feature
- fix: Bug fix
- docs: Documentation changes
- style: Code style changes (formatting)
- refactor: Code refactoring
- test: Adding or updating tests
- chore: Maintenance tasks

## Branching
- Use descriptive branch names
- Keep branches short-lived
- Delete merged branches

## Pull Requests
- Write clear PR descriptions
- Link related issues
- Request reviews from relevant team members`,
    adapters: [
      "antigravity",
      "gemini",
      "opencode",
      "cline",
      "claude-code",
      "codex",
    ] as AdapterType[],
  },
];

interface DashboardProps {
  onNavigate: (view: string) => void;
}

export function Dashboard({ onNavigate }: DashboardProps) {
  const { rules, fetchRules, createRule, isLoading } = useRulesStore();
  const { addToast } = useToast();
  const [lastSync, setLastSync] = useState<string | null>(null);

  const [previewOpen, setPreviewOpen] = useState(false);
  const [previewResult, setPreviewResult] = useState<SyncResult | null>(null);
  const [isPreviewing, setIsPreviewing] = useState(false);

  const [isSyncing, setIsSyncing] = useState(false);
  const [syncProgress, setSyncProgress] = useState({
    currentFile: "",
    currentFileIndex: 0,
    totalFiles: 0,
    completedFiles: [] as { path: string; success: boolean }[],
  });

  const [resultsOpen, setResultsOpen] = useState(false);
  const [syncResult, setSyncResult] = useState<SyncResult | null>(null);
  const [syncHistory, setSyncHistory] = useState<SyncHistoryEntry[]>([]);

  useEffect(() => {
    fetchRules();
    fetchSyncHistory();
  }, [fetchRules]);

  const fetchSyncHistory = async () => {
    try {
      const history = await api.sync.getHistory(5);
      setSyncHistory(history);
      if (history.length > 0) {
        const lastEntry = history[0];
        setLastSync(new Date(lastEntry.timestamp).toLocaleTimeString());
      }
    } catch {
      console.error("Failed to fetch sync history");
    }
  };

  const handleSyncClick = async () => {
    setIsPreviewing(true);
    try {
      const result = await api.sync.previewSync();
      setPreviewResult(result);
      setPreviewOpen(true);
    } catch (error) {
      addToast({
        title: "Preview Failed",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "error",
      });
    } finally {
      setIsPreviewing(false);
    }
  };

  const handleConfirmSync = async () => {
    setPreviewOpen(false);
    setIsSyncing(true);

    const totalFiles = previewResult?.filesWritten.length || 0;
    setSyncProgress({
      currentFile: "",
      currentFileIndex: 0,
      totalFiles,
      completedFiles: [],
    });

    try {
      const result = await api.sync.syncRules();
      setSyncResult(result);
      setResultsOpen(true);

      if (result.success) {
        setLastSync(new Date().toLocaleTimeString());
        fetchSyncHistory();
        addToast({
          title: "Sync Complete",
          description: `${result.filesWritten.length} files updated`,
          variant: "success",
        });
      } else {
        addToast({
          title: "Sync Completed with Issues",
          description: `${result.errors.length} errors occurred`,
          variant: "warning",
        });
      }
    } catch (error) {
      addToast({
        title: "Sync Error",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "error",
      });
    } finally {
      setIsSyncing(false);
      setSyncProgress({
        currentFile: "",
        currentFileIndex: 0,
        totalFiles: 0,
        completedFiles: [],
      });
    }
  };

  const globalRules = rules.filter((r) => r.scope === "global");
  const localRules = rules.filter((r) => r.scope === "local");
  const enabledRules = rules.filter((r) => r.enabled);

  const handleTemplateClick = async (template: Template) => {
    try {
      const input: CreateRuleInput = {
        name: template.name,
        content: template.content,
        scope: "global",
        enabledAdapters: template.adapters,
      };
      await createRule(input);
      addToast({
        title: "Template Applied",
        description: `"${template.name}" rule created successfully`,
        variant: "success",
      });
    } catch (error) {
      addToast({
        title: "Failed to Create Rule",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "error",
      });
    }
  };

  return (
    <>
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold">Dashboard</h1>
            <p className="text-muted-foreground">
              Manage your AI coding assistant rules in one place
            </p>
          </div>
          <div className="flex gap-2">
            <Button
              variant="outline"
              onClick={handleSyncClick}
              disabled={isPreviewing || isSyncing}
            >
              <RefreshCw
                className={`mr-2 h-4 w-4 ${isPreviewing || isSyncing ? "animate-spin" : ""}`}
              />
              Sync All
            </Button>
            <Button onClick={() => onNavigate("rules")}>
              <Plus className="mr-2 h-4 w-4" />
              New Rule
            </Button>
          </div>
        </div>

        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Total Rules</CardTitle>
              <FileText className="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{rules.length}</div>
              <p className="text-xs text-muted-foreground">{enabledRules.length} enabled</p>
            </CardContent>
          </Card>

          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Global Rules</CardTitle>
              <Globe className="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{globalRules.length}</div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Local Rules</CardTitle>
              <FolderOpen className="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{localRules.length}</div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Last Sync</CardTitle>
              <Clock className="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{lastSync || "Never"}</div>
            </CardContent>
          </Card>
        </div>

        {rules.length === 0 && !isLoading && (
          <Card className="border-dashed">
            <CardContent className="flex flex-col items-center justify-center py-12">
              <FileText className="h-12 w-12 text-muted-foreground mb-4" />
              <h3 className="text-lg font-semibold mb-2">No rules yet</h3>
              <p className="text-muted-foreground text-center mb-4">
                Create your first rule to start managing your AI assistant configurations
              </p>
              <Button onClick={() => onNavigate("rules")}>
                <Plus className="mr-2 h-4 w-4" />
                Create First Rule
              </Button>
            </CardContent>
          </Card>
        )}

        {rules.length > 0 && (
          <Card>
            <CardHeader>
              <CardTitle>Quick Start Templates</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="grid gap-3 md:grid-cols-2 lg:grid-cols-4">
                {TEMPLATES.map((template) => (
                  <button
                    key={template.name}
                    onClick={() => handleTemplateClick(template)}
                    className="flex items-center gap-3 rounded-md border p-3 text-left transition-colors hover:bg-accent focus:outline-none focus:ring-2 focus:ring-ring"
                  >
                    <div className="flex h-10 w-10 items-center justify-center rounded-md bg-primary/10 text-primary font-mono text-sm">
                      {template.icon}
                    </div>
                    <div className="flex-1 min-w-0">
                      <span className="text-sm font-medium block truncate">{template.name}</span>
                      <span className="text-xs text-muted-foreground">
                        {template.adapters.length} adapters
                      </span>
                    </div>
                  </button>
                ))}
              </div>
            </CardContent>
          </Card>
        )}

        {syncHistory.length > 0 && (
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <History className="h-5 w-5" />
                Recent Sync History
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-2">
                {syncHistory.map((entry) => (
                  <div
                    key={entry.id}
                    className="flex items-center justify-between py-2 border-b last:border-0"
                  >
                    <div className="flex items-center gap-3">
                      {entry.status === "success" ? (
                        <CheckCircle className="h-4 w-4 text-success" />
                      ) : entry.status === "partial" ? (
                        <AlertTriangle className="h-4 w-4 text-warning" />
                      ) : (
                        <XCircle className="h-4 w-4 text-destructive" />
                      )}
                      <div>
                        <p className="text-sm font-medium">
                          {entry.filesWritten} file{entry.filesWritten !== 1 ? "s" : ""} synced
                        </p>
                        <p className="text-xs text-muted-foreground">
                          {new Date(entry.timestamp).toLocaleString()}
                        </p>
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      <Badge
                        variant={
                          entry.status === "success"
                            ? "success"
                            : entry.status === "partial"
                              ? "warning"
                              : "destructive"
                        }
                      >
                        {entry.status}
                      </Badge>
                      <Badge variant="outline">{entry.triggeredBy}</Badge>
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        )}
      </div>

      <SyncPreviewDialog
        open={previewOpen}
        onOpenChange={setPreviewOpen}
        previewResult={previewResult}
        rules={rules}
        onConfirm={handleConfirmSync}
        onCancel={() => setPreviewOpen(false)}
        onConflictResolved={() => {
          fetchRules();
          handleSyncClick();
        }}
      />

      <SyncProgress
        isSyncing={isSyncing}
        currentFile={syncProgress.currentFile}
        currentFileIndex={syncProgress.currentFileIndex}
        totalFiles={syncProgress.totalFiles}
        completedFiles={syncProgress.completedFiles}
      />

      <SyncResultsDialog open={resultsOpen} onOpenChange={setResultsOpen} result={syncResult} />
    </>
  );
}
