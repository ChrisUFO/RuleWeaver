import { useEffect, useState, useCallback } from "react";
import { motion } from "framer-motion";
import {
  Activity,
  RefreshCw,
  Wrench,
  AlertCircle,
  CheckCircle,
  Clock,
  XCircle,
  Filter,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Select } from "@/components/ui/select";
import { useToast } from "@/components/ui/toast";
import { api } from "@/lib/tauri";
import { cn } from "@/lib/utils";
import type {
  ArtifactStatusEntry,
  ArtifactType,
  ArtifactSyncStatus,
  StatusFilter,
  StatusSummary,
} from "@/types/status";
import { ARTIFACT_TYPE_LABELS, SYNC_STATUS_CONFIG } from "@/types/status";
import { useRegistryStore } from "@/stores/registryStore";
import type { SelectOption } from "@/components/ui/select";
import type { AdapterType } from "@/types/rule";

const container = {
  hidden: { opacity: 0 },
  show: {
    opacity: 1,
    transition: { staggerChildren: 0.05 },
  },
};

const item = {
  hidden: { opacity: 0, y: 10 },
  show: { opacity: 1, y: 0 },
};

const ARTIFACT_TYPE_OPTIONS: SelectOption[] = [
  { value: "all", label: "All Types" },
  { value: "rule", label: "Rules" },
  { value: "command_stub", label: "Command Stubs" },
  { value: "slash_command", label: "Slash Commands" },
  { value: "skill", label: "Skills" },
];

const STATUS_OPTIONS: SelectOption[] = [
  { value: "all", label: "All Statuses" },
  { value: "synced", label: "Synced" },
  { value: "out_of_date", label: "Out of Date" },
  { value: "missing", label: "Missing" },
  { value: "conflicted", label: "Conflicted" },
  { value: "unsupported", label: "Unsupported" },
  { value: "error", label: "Error" },
];

export function Status() {
  const { addToast } = useToast();
  const { tools } = useRegistryStore();

  const [entries, setEntries] = useState<ArtifactStatusEntry[]>([]);
  const [summary, setSummary] = useState<StatusSummary | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isRepairing, setIsRepairing] = useState<string | null>(null);
  const [isBulkRepairing, setIsBulkRepairing] = useState(false);

  const [artifactTypeFilter, setArtifactTypeFilter] = useState("all");
  const [adapterFilter, setAdapterFilter] = useState("all");
  const [statusFilter, setStatusFilter] = useState("all");

  const adapterOptions: SelectOption[] = [
    { value: "all", label: "All Tools" },
    ...tools.map((tool) => ({ value: tool.id, label: tool.name })),
  ];

  const fetchStatus = useCallback(async () => {
    setIsLoading(true);
    try {
      const currentFilter: StatusFilter = {};
      if (artifactTypeFilter !== "all") {
        currentFilter.artifactType = artifactTypeFilter as ArtifactType;
      }
      if (adapterFilter !== "all") {
        currentFilter.adapter = adapterFilter as AdapterType;
      }
      if (statusFilter !== "all") {
        currentFilter.status = statusFilter as ArtifactSyncStatus;
      }

      const [statusEntries, statusSummary] = await Promise.all([
        api.status.getArtifactStatus(
          Object.keys(currentFilter).length > 0 ? currentFilter : undefined
        ),
        api.status.getSummary(Object.keys(currentFilter).length > 0 ? currentFilter : undefined),
      ]);

      setEntries(statusEntries);
      setSummary(statusSummary);
    } catch (error) {
      addToast({
        title: "Failed to load status",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "error",
      });
    } finally {
      setIsLoading(false);
    }
  }, [artifactTypeFilter, adapterFilter, statusFilter, addToast]);

  useEffect(() => {
    fetchStatus();
  }, [fetchStatus]);

  const handleRepair = async (entryId: string) => {
    setIsRepairing(entryId);
    try {
      const result = await api.status.repairArtifact(entryId);
      if (result.success) {
        addToast({
          title: "Repair successful",
          description: "Artifact has been repaired",
          variant: "success",
        });
        await fetchStatus();
      } else {
        addToast({
          title: "Repair failed",
          description: result.error || "Unknown error",
          variant: "error",
        });
      }
    } catch (error) {
      addToast({
        title: "Repair failed",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "error",
      });
    } finally {
      setIsRepairing(null);
    }
  };

  const handleBulkRepair = async () => {
    setIsBulkRepairing(true);
    try {
      const currentFilter: StatusFilter = {};
      if (artifactTypeFilter !== "all") {
        currentFilter.artifactType = artifactTypeFilter as ArtifactType;
      }
      if (adapterFilter !== "all") {
        currentFilter.adapter = adapterFilter as AdapterType;
      }
      if (statusFilter !== "all") {
        currentFilter.status = statusFilter as ArtifactSyncStatus;
      }

      const results = await api.status.repairAll(
        Object.keys(currentFilter).length > 0 ? currentFilter : undefined
      );
      const successCount = results.filter((r) => r.success).length;
      addToast({
        title: "Bulk repair complete",
        description: `${successCount} of ${results.length} artifacts repaired`,
        variant: successCount === results.length ? "success" : "warning",
      });
      await fetchStatus();
    } catch (error) {
      addToast({
        title: "Bulk repair failed",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "error",
      });
    } finally {
      setIsBulkRepairing(false);
    }
  };

  const getStatusIcon = (status: ArtifactSyncStatus) => {
    switch (status) {
      case "synced":
        return <CheckCircle className="h-4 w-4 text-green-500" />;
      case "out_of_date":
        return <Clock className="h-4 w-4 text-yellow-500" />;
      case "missing":
      case "conflicted":
        return <AlertCircle className="h-4 w-4 text-red-500" />;
      case "unsupported":
        return <XCircle className="h-4 w-4 text-gray-500" />;
      default:
        return <AlertCircle className="h-4 w-4 text-red-600" />;
    }
  };

  const nonSyncedCount = entries.filter(
    (e) => e.status !== "synced" && e.status !== "unsupported"
  ).length;

  return (
    <motion.div
      variants={container}
      initial="hidden"
      animate="show"
      className="space-y-6 max-w-7xl mx-auto"
    >
      <motion.div variants={item} className="flex items-end justify-between">
        <div className="space-y-1">
          <h1 className="text-4xl font-black tracking-tight luminescent-text">Artifact Status</h1>
          <p className="text-muted-foreground font-medium flex items-center gap-2">
            <Activity className="h-3 w-3 text-primary animate-pulse" />
            Unified sync health across all artifact types
          </p>
        </div>
        <div className="flex gap-3">
          <Button
            variant="outline"
            onClick={fetchStatus}
            disabled={isLoading}
            className="glass border-white/10 hover:bg-primary/5 transition-all duration-300"
          >
            <RefreshCw className={cn("mr-2 h-4 w-4", isLoading && "animate-spin")} />
            Refresh
          </Button>
          <Button
            onClick={handleBulkRepair}
            disabled={isLoading || isBulkRepairing || nonSyncedCount === 0}
            className="shadow-luminescent glow-primary"
          >
            <Wrench className={cn("mr-2 h-4 w-4", isBulkRepairing && "animate-spin")} />
            Repair All ({nonSyncedCount})
          </Button>
        </div>
      </motion.div>

      {summary && (
        <motion.div variants={item}>
          <div className="grid gap-4 md:grid-cols-4 lg:grid-cols-7">
            {[
              {
                label: "Total",
                value: summary.total,
                color: "text-blue-500",
                bgColor: "bg-blue-500/10",
              },
              {
                label: "Synced",
                value: summary.synced,
                color: "text-green-500",
                bgColor: "bg-green-500/10",
              },
              {
                label: "Out of Date",
                value: summary.outOfDate,
                color: "text-yellow-500",
                bgColor: "bg-yellow-500/10",
              },
              {
                label: "Missing",
                value: summary.missing,
                color: "text-red-500",
                bgColor: "bg-red-500/10",
              },
              {
                label: "Conflicted",
                value: summary.conflicted,
                color: "text-red-500",
                bgColor: "bg-red-500/10",
              },
              {
                label: "Unsupported",
                value: summary.unsupported,
                color: "text-gray-500",
                bgColor: "bg-gray-500/10",
              },
              {
                label: "Error",
                value: summary.error,
                color: "text-red-600",
                bgColor: "bg-red-600/10",
              },
            ].map((stat) => (
              <Card key={stat.label} className="glass-card border-none overflow-hidden">
                <CardContent className="p-4">
                  <div className="text-xs font-bold uppercase tracking-widest text-muted-foreground/60 mb-1">
                    {stat.label}
                  </div>
                  <div className={cn("text-2xl font-black", stat.color)}>{stat.value}</div>
                </CardContent>
              </Card>
            ))}
          </div>
        </motion.div>
      )}

      <motion.div variants={item}>
        <Card className="glass-card border-none">
          <CardHeader className="pb-3">
            <div className="flex items-center justify-between">
              <CardTitle className="text-lg font-bold flex items-center gap-2">
                <Filter className="h-4 w-4" />
                Filters
              </CardTitle>
            </div>
          </CardHeader>
          <CardContent>
            <div className="flex gap-4 flex-wrap">
              <div className="space-y-1">
                <label className="text-xs font-bold uppercase tracking-widest text-muted-foreground/60">
                  Type
                </label>
                <Select
                  options={ARTIFACT_TYPE_OPTIONS}
                  value={artifactTypeFilter}
                  onChange={setArtifactTypeFilter}
                  className="w-40"
                />
              </div>

              <div className="space-y-1">
                <label className="text-xs font-bold uppercase tracking-widest text-muted-foreground/60">
                  Tool
                </label>
                <Select
                  options={adapterOptions}
                  value={adapterFilter}
                  onChange={setAdapterFilter}
                  className="w-40"
                />
              </div>

              <div className="space-y-1">
                <label className="text-xs font-bold uppercase tracking-widest text-muted-foreground/60">
                  Status
                </label>
                <Select
                  options={STATUS_OPTIONS}
                  value={statusFilter}
                  onChange={setStatusFilter}
                  className="w-40"
                />
              </div>
            </div>
          </CardContent>
        </Card>
      </motion.div>

      <motion.div variants={item}>
        <Card className="glass-card border-none">
          <CardHeader>
            <CardTitle className="text-lg font-bold">
              Status Table
              <span className="ml-2 text-sm font-normal text-muted-foreground">
                ({entries.length} entries)
              </span>
            </CardTitle>
          </CardHeader>
          <CardContent>
            {isLoading ? (
              <div className="flex items-center justify-center py-12">
                <RefreshCw className="h-6 w-6 animate-spin text-primary" />
              </div>
            ) : entries.length === 0 ? (
              <div className="flex flex-col items-center justify-center py-12 text-center">
                <CheckCircle className="h-12 w-12 text-green-500 mb-4" />
                <h3 className="font-bold text-lg">All Synced</h3>
                <p className="text-sm text-muted-foreground">
                  All artifacts are in sync with the expected state.
                </p>
              </div>
            ) : (
              <div className="overflow-x-auto">
                <table className="w-full">
                  <thead>
                    <tr className="border-b border-white/10">
                      <th className="text-left py-3 px-4 text-xs font-bold uppercase tracking-widest text-muted-foreground/60">
                        Name
                      </th>
                      <th className="text-left py-3 px-4 text-xs font-bold uppercase tracking-widest text-muted-foreground/60">
                        Type
                      </th>
                      <th className="text-left py-3 px-4 text-xs font-bold uppercase tracking-widest text-muted-foreground/60">
                        Tool
                      </th>
                      <th className="text-left py-3 px-4 text-xs font-bold uppercase tracking-widest text-muted-foreground/60">
                        Scope
                      </th>
                      <th className="text-left py-3 px-4 text-xs font-bold uppercase tracking-widest text-muted-foreground/60">
                        Status
                      </th>
                      <th className="text-left py-3 px-4 text-xs font-bold uppercase tracking-widest text-muted-foreground/60">
                        Path
                      </th>
                      <th className="text-left py-3 px-4 text-xs font-bold uppercase tracking-widest text-muted-foreground/60">
                        Actions
                      </th>
                    </tr>
                  </thead>
                  <tbody>
                    {entries.map((entry) => (
                      <tr
                        key={entry.id}
                        className="border-b border-white/5 hover:bg-white/5 transition-colors"
                      >
                        <td className="py-3 px-4 font-medium">{entry.artifactName}</td>
                        <td className="py-3 px-4">
                          <Badge variant="outline" className="text-xs">
                            {ARTIFACT_TYPE_LABELS[entry.artifactType]}
                          </Badge>
                        </td>
                        <td className="py-3 px-4 text-sm text-muted-foreground">
                          {tools.find((t) => t.id === entry.adapter)?.name || entry.adapter}
                        </td>
                        <td className="py-3 px-4">
                          <Badge
                            variant="outline"
                            className={cn(
                              "text-xs",
                              entry.scope === "global"
                                ? "border-purple-500/20 text-purple-500"
                                : "border-emerald-500/20 text-emerald-500"
                            )}
                          >
                            {entry.scope}
                          </Badge>
                        </td>
                        <td className="py-3 px-4">
                          <div className="flex items-center gap-2">
                            {getStatusIcon(entry.status)}
                            <Badge
                              variant="outline"
                              className={cn("text-xs", SYNC_STATUS_CONFIG[entry.status]?.bgColor)}
                            >
                              {SYNC_STATUS_CONFIG[entry.status]?.label || entry.status}
                            </Badge>
                          </div>
                        </td>
                        <td className="py-3 px-4 text-xs font-mono text-muted-foreground max-w-xs truncate">
                          {entry.expectedPath}
                        </td>
                        <td className="py-3 px-4">
                          {entry.status !== "synced" && entry.status !== "unsupported" && (
                            <Button
                              size="sm"
                              variant="ghost"
                              onClick={() => handleRepair(entry.id)}
                              disabled={isRepairing === entry.id}
                              className="h-7 text-xs"
                            >
                              {isRepairing === entry.id ? (
                                <RefreshCw className="h-3 w-3 animate-spin" />
                              ) : (
                                <Wrench className="h-3 w-3 mr-1" />
                              )}
                              Repair
                            </Button>
                          )}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </CardContent>
        </Card>
      </motion.div>
    </motion.div>
  );
}
