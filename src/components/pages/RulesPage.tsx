import { useEffect, useState } from "react";
import { RulesList } from "./RulesList";
import { RuleEditor } from "./RuleEditor";
import type { Rule } from "@/types/rule";
import { useRulesStore } from "@/stores/rulesStore";

interface RulesPageProps {
  initialSelectedId?: string | null;
  onClearInitialId?: () => void;
}

export function RulesPage({ initialSelectedId, onClearInitialId }: RulesPageProps) {
  const { rules } = useRulesStore();
  const [selectedRule, setSelectedRule] = useState<Rule | null>(null);
  const [isNewRule, setIsNewRule] = useState(false);

  useEffect(() => {
    if (initialSelectedId && rules.length > 0) {
      const rule = rules.find((r) => r.id === initialSelectedId);
      if (rule) {
        setSelectedRule(rule);
        setIsNewRule(false);
        onClearInitialId?.();
      }
    }
  }, [initialSelectedId, rules, onClearInitialId]);

  const handleSelectRule = (rule: Rule) => {
    setSelectedRule(rule);
    setIsNewRule(false);
  };
  // ... (rest of component)

  const handleCreateRule = () => {
    setSelectedRule(null);
    setIsNewRule(true);
  };

  const handleBack = () => {
    setSelectedRule(null);
    setIsNewRule(false);
  };

  if (isNewRule) {
    return <RuleEditor rule={null} onBack={handleBack} isNew />;
  }

  if (selectedRule) {
    return <RuleEditor rule={selectedRule} onBack={handleBack} />;
  }

  return <RulesList onSelectRule={handleSelectRule} onCreateRule={handleCreateRule} />;
}
