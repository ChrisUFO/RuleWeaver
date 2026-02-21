import { useState } from "react";
import { RulesList } from "./RulesList";
import { RuleEditor } from "./RuleEditor";
import type { Rule } from "@/types/rule";

export function RulesPage() {
  const [selectedRule, setSelectedRule] = useState<Rule | null>(null);
  const [isNewRule, setIsNewRule] = useState(false);

  const handleSelectRule = (rule: Rule) => {
    setSelectedRule(rule);
    setIsNewRule(false);
  };

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
