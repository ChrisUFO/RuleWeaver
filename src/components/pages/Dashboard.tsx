import { useCallback, useEffect, useMemo, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { motion, AnimatePresence, type Variants } from "framer-motion";
import { cn } from "@/lib/utils";
import {
  Plus,
  RefreshCw,
  FileText,
  Terminal,
  Brain,
  CheckCircle,
  History,
  Activity,
  ShieldAlert,
  AlertTriangle,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { useRulesStore } from "@/stores/rulesStore";
import { useToast } from "@/components/ui/toast";
import { api } from "@/lib/tauri";
import { SyncPreviewDialog } from "@/components/sync/SyncPreviewDialog";
import { SyncProgress } from "@/components/sync/SyncProgress";
import { SyncResultsDialog } from "@/components/sync/SyncResultsDialog";
import { DashboardSkeleton } from "@/components/ui/skeleton";
import { buildDashboardMetrics, emptyDashboardMetrics } from "@/lib/dashboard-metrics";
import { applySyncProgressEvent, EMPTY_SYNC_PROGRESS_STATE } from "@/lib/sync-progress";
import type { SyncResult, SyncHistoryEntry, SyncProgressEvent } from "@/types/rule";

const container: Variants = {
  hidden: { opacity: 0 },
  show: {
    opacity: 1,
    transition: {
      staggerChildren: 0.1,
    },
  },
};

const item: Variants = {
  hidden: { opacity: 0, y: 20 },
  show: {
    opacity: 1,
    y: 0,
    transition: {
      type: "spring",
      stiffness: 300,
      damping: 24,
    },
  },
};

export function Dashboard({ onNavigate }: { onNavigate: (view: string, id?: string) => void }) {
  const { rules, fetchRules, isLoading } = useRulesStore();
  const { addToast } = useToast();
  const [lastSync, setLastSync] = useState<string | null>(null);
  const [metrics, setMetrics] = useState(emptyDashboardMetrics());
  const [isLoadingMetrics, setIsLoadingMetrics] = useState(true);
  const [metricsError, setMetricsError] = useState<string | null>(null);

  const [previewOpen, setPreviewOpen] = useState(false);
  const [previewResult, setPreviewResult] = useState<SyncResult | null>(null);
  const [isPreviewing, setIsPreviewing] = useState(false);

  const [isSyncing, setIsSyncing] = useState(false);
  const [syncProgress, setSyncProgress] = useState(EMPTY_SYNC_PROGRESS_STATE);

  const [resultsOpen, setResultsOpen] = useState(false);
  const [syncResult, setSyncResult] = useState<SyncResult | null>(null);
  const [syncHistory, setSyncHistory] = useState<SyncHistoryEntry[]>([]);
  const [fullHistoryOpen, setFullHistoryOpen] = useState(false);
  const [isLoadingFullHistory, setIsLoadingFullHistory] = useState(false);
  const [fullSyncHistory, setFullSyncHistory] = useState<SyncHistoryEntry[]>([]);
  const [fullHistoryFilter, setFullHistoryFilter] = useState<
    "all" | "success" | "partial" | "failed"
  >("all");
  const [fullHistoryError, setFullHistoryError] = useState<string | null>(null);

  const refreshDashboardMetrics = useCallback(async () => {
    setIsLoadingMetrics(true);
    setMetricsError(null);
    try {
      const [rulesData, commands, skills, statusEntries] = await Promise.all([
        api.rules.getAll(),
        api.commands.getAll(),
        api.skills.getAll(),
        api.status.getArtifactStatus(),
      ]);

      setMetrics(
        buildDashboardMetrics({
          rules: rulesData,
          commands,
          skills,
          statusEntries,
        })
      );
    } catch (error) {
      const message = error instanceof Error ? error.message : "Failed to load dashboard metrics";
      setMetricsError(message);
      addToast({ title: "Dashboard Metrics Unavailable", description: message, variant: "error" });
    } finally {
      setIsLoadingMetrics(false);
    }
  }, [addToast]);

  useEffect(() => {
    fetchRules();
    fetchSyncHistory();
    refreshDashboardMetrics();
  }, [fetchRules, refreshDashboardMetrics]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let cancelled = false;

    const bindSyncProgressListener = async () => {
      const listener = await listen<SyncProgressEvent>("sync-progress", (event) => {
        if (event.payload.phase === "start") {
          setIsSyncing(true);
        }
        if (event.payload.phase === "complete" || event.payload.phase === "error") {
          setIsSyncing(false);
        }
        setSyncProgress((previous) => applySyncProgressEvent(previous, event.payload));
      });

      if (cancelled) {
        listener();
        return;
      }

      unlisten = listener;
    };

    bindSyncProgressListener().catch((error) => {
      console.error("Failed to bind sync progress listener", error);
    });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  const fetchSyncHistory = async () => {
    try {
      const history = await api.sync.getHistory(5);
      setSyncHistory(history);
      if (history.length > 0) {
        const lastEntry = history[0];
        setLastSync(new Date(lastEntry.timestamp * 1000).toLocaleTimeString());
      }
    } catch {
      console.error("Failed to fetch sync history");
    }
  };

  const handleViewFullLogs = async () => {
    setIsLoadingFullHistory(true);
    setFullHistoryError(null);
    try {
      const history = await api.sync.getHistory(100);
      setFullSyncHistory(history);
      setFullHistoryOpen(true);
    } catch (error) {
      const message = error instanceof Error ? error.message : "Unknown error";
      setFullHistoryError(message);
      setFullHistoryOpen(true);
      addToast({ title: "Failed to Load Logs", description: message, variant: "error" });
    } finally {
      setIsLoadingFullHistory(false);
    }
  };

  const filteredFullHistory = fullSyncHistory.filter((entry) =>
    fullHistoryFilter === "all" ? true : entry.status === fullHistoryFilter
  );

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
    setSyncProgress({ ...EMPTY_SYNC_PROGRESS_STATE, totalFiles });

    try {
      const result = await api.sync.syncRules();
      setSyncResult(result);
      setResultsOpen(true);

      if (result.success) {
        setLastSync(new Date().toLocaleTimeString());
        await Promise.all([fetchSyncHistory(), refreshDashboardMetrics()]);
        addToast({
          title: "Sync Complete",
          description: `${(result.filesWritten || []).length} files updated`,
          variant: "success",
        });
      } else {
        await refreshDashboardMetrics();
        addToast({
          title: "Sync Completed with Issues",
          description: `${(result.errors || []).length} errors occurred`,
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
      setSyncProgress(EMPTY_SYNC_PROGRESS_STATE);
    }
  };

  const healthState = useMemo<"loading" | "error" | "attention" | "synced">(() => {
    if (isLoadingMetrics) {
      return "loading";
    }
    if (metricsError) {
      return "error";
    }
    return metrics.overallAttention > 0 ? "attention" : "synced";
  }, [isLoadingMetrics, metricsError, metrics.overallAttention]);

  const metricCards = [
    {
      label: "Total Artifacts",
      value: metrics.totalArtifacts,
      sub: `${metrics.activeArtifacts} Active`,
      icon: FileText,
      color: "text-blue-500",
    },
    {
      label: "Rules",
      value: metrics.rules.count,
      sub:
        metrics.rules.health.tracked > 0
          ? metrics.rules.health.attention > 0
            ? `${metrics.rules.health.attention} Need Attention`
            : "All Synchronized"
          : "No Status Data",
      icon: FileText,
      color: "text-emerald-500",
    },
    {
      label: "Commands",
      value: metrics.commands.count,
      sub:
        metrics.commands.health.tracked > 0
          ? metrics.commands.health.attention > 0
            ? `${metrics.commands.health.attention} Need Attention`
            : "All Synchronized"
          : "No Status Data",
      icon: Terminal,
      color: "text-amber-500",
    },
    {
      label: "Skills",
      value: metrics.skills.count,
      sub:
        metrics.skills.health.tracked > 0
          ? metrics.skills.health.attention > 0
            ? `${metrics.skills.health.attention} Need Attention`
            : "All Synchronized"
          : "No Status Data",
      icon: Brain,
      color: "text-cyan-500",
    },
  ];

  return (
    <>
      <AnimatePresence mode="wait">
        {isLoading || isLoadingMetrics ? (
          <motion.div
            key="skeleton"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.3 }}
          >
            <DashboardSkeleton />
          </motion.div>
        ) : (
          <motion.div
            key="content"
            variants={container}
            initial="hidden"
            animate="show"
            exit={{ opacity: 0 }}
            className="space-y-8 max-w-7xl mx-auto pb-12"
          >
            {/* Header Section */}
            <motion.div variants={item} className="flex items-end justify-between">
              <div className="space-y-1">
                <h1 className="text-4xl font-black tracking-tight luminescent-text">
                  Command Center
                </h1>
                <p className="text-muted-foreground font-medium flex items-center gap-2">
                  <Activity className="h-3 w-3 text-primary animate-pulse" />
                  System operational. {metrics.totalArtifacts} artifacts monitored.
                </p>
              </div>
              <div className="flex gap-3">
                <Button
                  variant="outline"
                  onClick={handleSyncClick}
                  disabled={isPreviewing || isSyncing}
                  className="glass border-white/10 hover:bg-primary/5 transition-all duration-300"
                >
                  <RefreshCw
                    className={`mr-2 h-4 w-4 ${isPreviewing || isSyncing ? "animate-spin" : ""}`}
                  />
                  System Audit
                </Button>
                <Button
                  onClick={() => onNavigate("rules")}
                  className="shadow-luminescent glow-primary"
                >
                  <Plus className="mr-2 h-4 w-4" />
                  New Artifact
                </Button>
              </div>
            </motion.div>

            {/* Vital Stats Grid */}
            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
              {metricCards.map((stat) => (
                <motion.div key={stat.label} variants={item}>
                  <Card className="glass-card border-none overflow-hidden group hover:shadow-glow-primary transition-all duration-500">
                    <div className="absolute top-0 left-0 w-1 h-full bg-primary/20 group-hover:bg-primary transition-colors" />
                    <CardHeader className="flex flex-row items-center justify-between pb-2">
                      <span className="text-xs font-bold uppercase tracking-widest text-muted-foreground/60">
                        {stat.label}
                      </span>
                      <stat.icon
                        className={`h-4 w-4 ${stat.color} opacity-80 group-hover:scale-110 transition-transform`}
                      />
                    </CardHeader>
                    <CardContent>
                      <div className="text-3xl font-black group-hover:luminescent-text transition-all">
                        {stat.value}
                      </div>
                      <div className="text-[10px] font-bold uppercase tracking-tighter text-muted-foreground/40 mt-1">
                        {stat.sub}
                      </div>
                    </CardContent>
                  </Card>
                </motion.div>
              ))}
            </div>

            {/* Main Content Area */}
            <div className="grid gap-6 lg:grid-cols-3">
              {/* Health Monitor */}
              <motion.div variants={item} className="lg:col-span-2">
                <Card className="glass-card h-full bg-card/20 group overflow-hidden flex flex-col">
                  <CardHeader className="flex flex-row items-center justify-between">
                    <CardTitle className="text-lg font-bold flex items-center gap-2">
                      <ShieldAlert
                        className={cn(
                          "h-5 w-5",
                          healthState === "error"
                            ? "text-destructive"
                            : healthState === "attention"
                              ? "text-amber-500"
                              : "text-primary"
                        )}
                      />
                      Health Monitor
                    </CardTitle>
                    <div className="flex items-center gap-2">
                      {isLoadingMetrics && (
                        <Activity className="h-3 w-3 text-primary animate-pulse" />
                      )}
                      <Badge
                        variant="outline"
                        className="text-[10px] border-white/5 py-0 px-2 uppercase font-black text-muted-foreground/60"
                      >
                        Live View
                      </Badge>
                    </div>
                  </CardHeader>
                  <CardContent className="h-[240px] flex-1 flex flex-col items-center justify-center text-center space-y-4">
                    <div className="relative">
                      <div
                        className={cn(
                          "h-32 w-32 rounded-full border-4 flex items-center justify-center relative transition-colors duration-500",
                          healthState === "attention" ? "border-amber-500/20" : "border-primary/20"
                        )}
                      >
                        <div
                          className={cn(
                            "h-24 w-24 rounded-full border-4 flex items-center justify-center animate-pulse transition-colors duration-500",
                            healthState === "attention"
                              ? "border-amber-500/10"
                              : "border-primary/10"
                          )}
                        >
                          {healthState === "error" ? (
                            <AlertTriangle className="h-12 w-12 text-destructive/80" />
                          ) : healthState === "attention" ? (
                            <ShieldAlert className="h-12 w-12 text-amber-500/80" />
                          ) : (
                            <CheckCircle className="h-12 w-12 text-primary/80" />
                          )}
                        </div>
                      </div>
                    </div>
                    <div>
                      <h3 className="font-bold text-lg">
                        {healthState === "loading"
                          ? "Initializing Health View..."
                          : healthState === "error"
                            ? "Health Data Unavailable"
                            : healthState === "attention"
                              ? "Attention Required"
                              : "System Synchronized"}
                      </h3>
                      <p className="text-sm text-muted-foreground max-w-sm mx-auto">
                        {healthState === "loading"
                          ? "Collecting artifact status across rules, commands, and skills..."
                          : healthState === "error"
                            ? "Unable to load artifact health right now. Sync history and actions remain available."
                            : healthState === "attention"
                              ? `${metrics.overallAttention} managed artifacts need attention.`
                              : `All ${metrics.overallTrackedStatus} tracked artifacts are synchronized.`}
                      </p>
                      <p className="text-[10px] font-bold uppercase tracking-widest text-muted-foreground/50 mt-3">
                        Last sync: {lastSync || "N/A"}
                      </p>
                    </div>
                  </CardContent>
                </Card>
              </motion.div>

              {/* Sync History Container */}
              <motion.div variants={item}>
                <Card className="glass-card h-full bg-card/20 flex flex-col overflow-hidden">
                  <CardHeader>
                    <CardTitle className="text-sm font-bold uppercase tracking-widest text-muted-foreground/60 flex items-center gap-2">
                      <History className="h-4 w-4" />
                      Audit Trail
                    </CardTitle>
                  </CardHeader>
                  <CardContent className="flex-1 overflow-auto">
                    <div className="space-y-4">
                      {syncHistory.length > 0 ? (
                        syncHistory.map((entry) => (
                          <div key={entry.id} className="group relative pl-4">
                            <div className="absolute left-0 top-1 bottom-1 w-1 bg-white/5 rounded-full" />
                            <div className="flex items-center justify-between mb-1">
                              <span className="text-xs font-bold text-foreground/80">
                                {entry.filesWritten} Artifacts Updated
                              </span>
                              <span className="text-[10px] text-muted-foreground font-mono">
                                {new Date(entry.timestamp * 1000).toLocaleTimeString([], {
                                  hour: "2-digit",
                                  minute: "2-digit",
                                })}
                              </span>
                            </div>
                            <div className="flex items-center gap-2">
                              <Badge
                                className="h-4 text-[9px] uppercase font-black px-1.5"
                                variant={
                                  entry.status === "partial"
                                    ? "warning"
                                    : entry.status === "failed"
                                      ? "destructive"
                                      : "success"
                                }
                              >
                                {entry.status}
                              </Badge>
                              <span className="text-[10px] text-muted-foreground/60 uppercase font-black">
                                Via {entry.triggeredBy}
                              </span>
                            </div>
                          </div>
                        ))
                      ) : (
                        <div className="h-full flex items-center justify-center text-center p-8">
                          <p className="text-[10px] font-bold uppercase tracking-widest text-muted-foreground/40">
                            No audit history available
                          </p>
                        </div>
                      )}
                    </div>
                  </CardContent>
                  <div className="p-4 bg-white/5 mt-auto">
                    <Button
                      variant="ghost"
                      onClick={handleViewFullLogs}
                      disabled={isLoadingFullHistory || syncHistory.length === 0}
                      title={
                        syncHistory.length === 0
                          ? "Run a sync to generate logs"
                          : "Open full sync log history"
                      }
                      className="w-full h-8 text-xs font-bold text-primary/60 hover:text-primary hover:bg-transparent"
                    >
                      {isLoadingFullHistory ? "Loading Logs..." : "View Full Logs"}
                    </Button>
                    {syncHistory.length === 0 && (
                      <p className="mt-2 text-[10px] font-bold uppercase tracking-wider text-muted-foreground/60 text-center">
                        Run a sync to generate logs
                      </p>
                    )}
                  </div>
                </Card>
              </motion.div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>

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

      <Dialog open={fullHistoryOpen} onOpenChange={setFullHistoryOpen}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>Sync Logs</DialogTitle>
          </DialogHeader>

          <div className="flex flex-wrap items-center justify-between gap-3">
            <div className="flex items-center gap-2">
              {(
                [
                  ["all", "All"],
                  ["success", "Success"],
                  ["partial", "Partial"],
                  ["failed", "Failed"],
                ] as const
              ).map(([value, label]) => (
                <Button
                  key={value}
                  size="sm"
                  variant={fullHistoryFilter === value ? "default" : "outline"}
                  onClick={() => setFullHistoryFilter(value)}
                  className="h-7 text-[10px] uppercase font-black tracking-wider"
                >
                  {label}
                </Button>
              ))}
            </div>
            <p className="text-xs text-muted-foreground">
              Showing {filteredFullHistory.length} of {fullSyncHistory.length}
            </p>
          </div>

          <div className="max-h-[50vh] overflow-auto space-y-2 pr-1">
            {fullHistoryError ? (
              <div className="rounded border border-destructive/30 bg-destructive/10 p-4 space-y-3">
                <p className="text-sm font-semibold">Unable to load sync logs</p>
                <p className="text-xs text-muted-foreground break-all">{fullHistoryError}</p>
                <Button size="sm" variant="outline" onClick={handleViewFullLogs}>
                  Retry
                </Button>
              </div>
            ) : fullSyncHistory.length === 0 ? (
              <p className="text-sm text-muted-foreground">No sync logs available yet.</p>
            ) : filteredFullHistory.length === 0 ? (
              <p className="text-sm text-muted-foreground">No logs match this filter.</p>
            ) : (
              filteredFullHistory.map((entry) => (
                <div key={entry.id} className="rounded border border-white/10 p-3">
                  <div className="flex items-center justify-between text-xs">
                    <span className="font-semibold">
                      {entry.filesWritten} artifact{entry.filesWritten === 1 ? "" : "s"} updated
                    </span>
                    <span className="text-muted-foreground">
                      {new Date(entry.timestamp * 1000).toLocaleString()}
                    </span>
                  </div>
                  <div className="mt-1 flex items-center gap-2">
                    <Badge
                      variant={
                        entry.status === "partial"
                          ? "warning"
                          : entry.status === "failed"
                            ? "destructive"
                            : "success"
                      }
                    >
                      {entry.status}
                    </Badge>
                    <span className="text-xs text-muted-foreground uppercase">
                      Triggered by {entry.triggeredBy}
                    </span>
                  </div>
                </div>
              ))
            )}
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={() => setFullHistoryOpen(false)}>
              Close
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}
