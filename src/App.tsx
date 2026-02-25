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
import { motion, AnimatePresence } from "framer-motion";
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
      setActiveConflict({
        id: crypto.randomUUID(),
        filePath,
        adapterName: "Detected Adapter",
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
    const views: Record<string, React.ReactNode> = {
      dashboard: <Dashboard onNavigate={setActiveView} />,
      rules: <RulesPage />,
      commands: <Commands />,
      skills: <Skills />,
      settings: <Settings />,
    };

    const currentViewComponent = views[activeView] || <Dashboard onNavigate={setActiveView} />;

    return (
      <AnimatePresence mode="wait">
        <motion.div
          key={activeView}
          initial={{ opacity: 0, y: 10, filter: "blur(10px)" }}
          animate={{ opacity: 1, y: 0, filter: "blur(0px)" }}
          exit={{ opacity: 0, y: -10, filter: "blur(10px)" }}
          transition={{ duration: 0.3, ease: [0.4, 0, 0.2, 1] }}
          className="h-full"
        >
          {currentViewComponent}
        </motion.div>
      </AnimatePresence>
    );
  };

  return (
    <ToastProvider>
      <ErrorBoundary>
        <MainLayout activeView={activeView} onViewChange={setActiveView}>
          <div className="h-full relative overflow-hidden">
            {/* Ambient Background Glow */}
            <div className="absolute top-[-10%] right-[-10%] w-[40%] h-[40%] bg-primary/5 blur-[120px] rounded-full animate-luminescence pointer-events-none" />
            <div className="absolute bottom-[-5%] left-[-5%] w-[30%] h-[30%] bg-primary/5 blur-[100px] rounded-full animate-luminescence pointer-events-none [animation-delay:2s]" />

            {renderContent()}
          </div>
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
