import { ShieldCheck, RotateCcw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";

interface MigrationProgress {
  total: number;
  migrated: number;
  current_rule?: string;
  status: "NotStarted" | "InProgress" | "Completed" | "Failed" | "RolledBack";
}

interface StorageSettingsCardProps {
  storageMode: "sqlite" | "file";
  storageInfo: Record<string, string> | null;
  isMigratingStorage: boolean;
  backupPath: string;
  migrationProgress: MigrationProgress | null;
  isRollingBack: boolean;
  isVerifyingMigration: boolean;
  isLoading: boolean;
  onMigrate: () => Promise<void>;
  onRollback: () => Promise<void>;
  onVerify: () => Promise<void>;
}

export function StorageSettingsCard({
  storageMode,
  storageInfo,
  isMigratingStorage,
  backupPath,
  migrationProgress,
  isRollingBack,
  isVerifyingMigration,
  isLoading,
  onMigrate,
  onRollback,
  onVerify,
}: StorageSettingsCardProps) {
  return (
    <Card className="glass-card premium-shadow border-none overflow-hidden">
      <CardHeader className="bg-white/5 pb-4">
        <CardTitle className="text-sm font-semibold tracking-wide uppercase text-muted-foreground/80">
          Storage
        </CardTitle>
        <CardDescription>
          Manage where rules are stored: legacy SQLite or file-based markdown storage
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4 pt-6">
        <div className="flex items-center justify-between rounded-md border p-3">
          <div>
            <div className="font-medium">Current Mode</div>
            <div className="text-sm text-muted-foreground">
              {storageMode === "file"
                ? "File storage (.ruleweaver/rules/*.md)"
                : "SQLite database (legacy)"}
            </div>
          </div>
          <Badge variant={storageMode === "file" ? "default" : "outline"}>
            {storageMode === "file" ? "File" : "SQLite"}
          </Badge>
        </div>

        {storageInfo && (
          <div className="grid grid-cols-1 gap-2 text-sm text-muted-foreground md:grid-cols-3">
            <div>Rules: {storageInfo.rule_count ?? "0"}</div>
            <div>Size: {storageInfo.total_size_bytes ?? "0"} bytes</div>
            <div>Storage Exists: {storageInfo.exists ?? "false"}</div>
          </div>
        )}

        {storageMode !== "file" && (
          <Button onClick={onMigrate} disabled={isMigratingStorage || isLoading}>
            {isMigratingStorage ? "Migrating..." : "Migrate to File Storage"}
          </Button>
        )}

        {migrationProgress && (
          <div className="rounded-md border p-3 space-y-2">
            <div className="flex items-center justify-between text-sm">
              <span className="text-muted-foreground">Migration Status</span>
              <Badge variant="outline">{migrationProgress.status}</Badge>
            </div>
            <div className="text-sm">
              {migrationProgress.migrated} / {migrationProgress.total || 0} rules migrated
            </div>
            {migrationProgress.current_rule && (
              <div className="text-xs text-muted-foreground truncate">
                Current: {migrationProgress.current_rule}
              </div>
            )}
          </div>
        )}

        {storageMode === "file" && (
          <div className="flex flex-wrap gap-2">
            <Button variant="outline" onClick={onVerify} disabled={isVerifyingMigration}>
              <ShieldCheck className="mr-2 h-4 w-4" />
              {isVerifyingMigration ? "Verifying..." : "Verify Migration"}
            </Button>
            <Button variant="outline" onClick={onRollback} disabled={isRollingBack || !backupPath}>
              <RotateCcw className="mr-2 h-4 w-4" />
              {isRollingBack ? "Rolling Back..." : "Rollback"}
            </Button>
          </div>
        )}

        {backupPath && (
          <p className="text-xs text-muted-foreground break-all">Backup: {backupPath}</p>
        )}
      </CardContent>
    </Card>
  );
}
