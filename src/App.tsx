import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { MainLayout } from "./components/layout/MainLayout";
import { Dashboard } from "./components/pages/Dashboard";
import { RulesPage } from "./components/pages/RulesPage";
import { Commands } from "./components/pages/Commands";
import { Skills } from "./components/pages/Skills";
import { Settings } from "./components/pages/Settings";
import { ToastProvider } from "./components/ui/toast";
import { ErrorBoundary } from "./components/ui/error-boundary";
import { KeyboardShortcutsDialog } from "./components/ui/keyboard-shortcuts-dialog";
import { ConflictResolutionDialog } from "./components/sync/ConflictResolutionDialog";
import { useKeyboardShortcuts, SHORTCUTS } from "./hooks/useKeyboardShortcuts";
import { useRulesStore } from "./stores/rulesStore";
import type { Conflict } from "./types/rule";
import "./index.css";

function App() {
  const [activeView, setActiveView] = useState("dashboard");
  const [shortcutsDialogOpen, setShortcutsDialogOpen] = useState(false);

  const { rules, fetchRules } = useRulesStore();
  const [activeConflict, setActiveConflict] = useState<Conflict | null>(null);
  const [isConflictDialogOpen, setIsConflictDialogOpen] = useState(false);

  useEffect(() => {
    fetchRules();

    const unlisten = listen<string>("rule-conflict", async (event) => {
      const filePath = event.payload;
      // We need to construct a partial conflict object or fetch it from a sync preview
      // For now, let's assume the conflict dialog can handle a simple path
      setActiveConflict({
        id: crypto.randomUUID(),
        filePath,
        adapterName: "Detected Adapter", // Will be refined in dialog
        localHash: "",
        currentHash: "",
      });
      setIsConflictDialogOpen(true);
    });

    return () => {
      unlisten.then((f) => f());
    };
  }, [fetchRules]);

  useKeyboardShortcuts({
    shortcuts: [
      {
        ...SHORTCUTS.NEW_RULE,
        action: () => setActiveView("rules"),
      },
      {
        ...SHORTCUTS.SETTINGS,
        action: () => setActiveView("settings"),
      },
      {
        ...SHORTCUTS.NEW_COMMAND,
        action: () => setActiveView("commands"),
      },
      {
        ...SHORTCUTS.DASHBOARD,
        action: () => setActiveView("dashboard"),
      },
      {
        ...SHORTCUTS.RULES,
        action: () => setActiveView("rules"),
      },
      {
        ...SHORTCUTS.COMMANDS,
        action: () => setActiveView("commands"),
      },
      {
        ...SHORTCUTS.SKILLS,
        action: () => setActiveView("skills"),
      },
      {
        ...SHORTCUTS.HELP,
        action: () => setShortcutsDialogOpen(true),
      },
    ],
  });

  const renderContent = () => {
    switch (activeView) {
      case "dashboard":
        return <Dashboard onNavigate={setActiveView} />;
      case "rules":
        return <RulesPage />;
      case "commands":
        return <Commands />;
      case "skills":
        return <Skills />;
      case "settings":
        return <Settings />;
      default:
        return <Dashboard onNavigate={setActiveView} />;
    }
  };

  return (
    <ToastProvider>
      <ErrorBoundary>
        <MainLayout activeView={activeView} onViewChange={setActiveView}>
          {renderContent()}
        </MainLayout>
        <KeyboardShortcutsDialog open={shortcutsDialogOpen} onOpenChange={setShortcutsDialogOpen} />
        <ConflictResolutionDialog
          open={isConflictDialogOpen}
          onOpenChange={setIsConflictDialogOpen}
          conflict={activeConflict}
          localRules={rules}
          onResolved={() => {
            fetchRules();
            setIsConflictDialogOpen(false);
          }}
        />
      </ErrorBoundary>
    </ToastProvider>
  );
}

export default App;
