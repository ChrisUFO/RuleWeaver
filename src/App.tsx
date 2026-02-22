import { useState } from "react";
import { MainLayout } from "./components/layout/MainLayout";
import { Dashboard } from "./components/pages/Dashboard";
import { RulesPage } from "./components/pages/RulesPage";
import { Commands } from "./components/pages/Commands";
import { Skills } from "./components/pages/Skills";
import { Settings } from "./components/pages/Settings";
import { ToastProvider } from "./components/ui/toast";
import { ErrorBoundary } from "./components/ui/error-boundary";
import { KeyboardShortcutsDialog } from "./components/ui/keyboard-shortcuts-dialog";
import { useKeyboardShortcuts, SHORTCUTS } from "./hooks/useKeyboardShortcuts";
import "./index.css";

function App() {
  const [activeView, setActiveView] = useState("dashboard");
  const [shortcutsDialogOpen, setShortcutsDialogOpen] = useState(false);

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
      </ErrorBoundary>
    </ToastProvider>
  );
}

export default App;
