import { useEffect, useMemo, useState } from "react";
import { Plus, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import { api } from "@/lib/tauri";
import { useToast } from "@/components/ui/toast";
import type { Skill } from "@/types/skill";

export function Skills() {
  const [skills, setSkills] = useState<Skill[]>([]);
  const [selectedId, setSelectedId] = useState("");
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [instructions, setInstructions] = useState("");
  const [enabled, setEnabled] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const { addToast } = useToast();

  const selected = useMemo(
    () => skills.find((s) => s.id === selectedId) ?? null,
    [skills, selectedId]
  );

  const loadSkills = async () => {
    const data = await api.skills.getAll();
    setSkills(data);
  };

  useEffect(() => {
    loadSkills().catch((error) => {
      addToast({
        title: "Failed to Load Skills",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "error",
      });
    });
  }, [addToast]);

  useEffect(() => {
    if (!selected) {
      setName("");
      setDescription("");
      setInstructions("");
      setEnabled(true);
      return;
    }
    setName(selected.name);
    setDescription(selected.description);
    setInstructions(selected.instructions);
    setEnabled(selected.enabled);
  }, [selected]);

  const createSkill = async () => {
    setIsSaving(true);
    try {
      const created = await api.skills.create({
        name: "New Skill",
        description: "Describe this workflow",
        instructions: "Step 1\nStep 2",
        enabled: true,
      });
      await loadSkills();
      setSelectedId(created.id);
      addToast({ title: "Skill Created", description: created.name, variant: "success" });
    } catch (error) {
      addToast({
        title: "Create Failed",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "error",
      });
    } finally {
      setIsSaving(false);
    }
  };

  const saveSkill = async () => {
    if (!selected) return;
    setIsSaving(true);
    try {
      const updated = await api.skills.update(selected.id, {
        name,
        description,
        instructions,
        enabled,
      });
      setSkills((prev) => prev.map((s) => (s.id === updated.id ? updated : s)));
      addToast({ title: "Skill Saved", description: updated.name, variant: "success" });
    } catch (error) {
      addToast({
        title: "Save Failed",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "error",
      });
    } finally {
      setIsSaving(false);
    }
  };

  const deleteSkill = async () => {
    if (!selected) return;
    setIsSaving(true);
    try {
      await api.skills.delete(selected.id);
      setSkills((prev) => prev.filter((s) => s.id !== selected.id));
      setSelectedId("");
      addToast({ title: "Skill Deleted", description: selected.name, variant: "success" });
    } catch (error) {
      addToast({
        title: "Delete Failed",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "error",
      });
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <div className="grid gap-6 lg:grid-cols-[320px,1fr]">
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <CardTitle>Skills</CardTitle>
            <Button size="sm" onClick={createSkill} disabled={isSaving}>
              <Plus className="mr-2 h-4 w-4" />
              New
            </Button>
          </div>
          <CardDescription>Complex multi-step workflows</CardDescription>
        </CardHeader>
        <CardContent className="space-y-2">
          {skills.map((skill) => (
            <button
              key={skill.id}
              className={`w-full rounded-md border px-3 py-2 text-left transition ${
                selectedId === skill.id ? "border-primary bg-accent" : "hover:bg-accent"
              }`}
              onClick={() => setSelectedId(skill.id)}
            >
              <div className="font-medium truncate">{skill.name}</div>
              <div className="text-xs text-muted-foreground truncate">{skill.description}</div>
            </button>
          ))}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>{selected ? "Edit Skill" : "Select a Skill"}</CardTitle>
          <CardDescription>Define reusable instructions and workflow context.</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="rounded-md border border-amber-300 bg-amber-50 p-3 text-xs text-amber-900">
            Security warning: Skills execute shell commands with your current user privileges. Treat
            imported or shared skills as trusted code only.
          </div>
          {!selected ? (
            <p className="text-sm text-muted-foreground">Select a skill or create a new one.</p>
          ) : (
            <>
              <Input
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="Skill name"
              />
              <Input
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                placeholder="Description"
              />
              <textarea
                value={instructions}
                onChange={(e) => setInstructions(e.target.value)}
                className="min-h-48 rounded-md border bg-background p-3 text-sm"
                placeholder="Write workflow instructions..."
              />
              <div className="flex items-center justify-between rounded-md border p-3">
                <span className="text-sm">Enabled</span>
                <Switch checked={enabled} onCheckedChange={setEnabled} />
              </div>
              <div className="flex gap-2">
                <Button onClick={saveSkill} disabled={isSaving}>
                  {isSaving ? "Saving..." : "Save"}
                </Button>
                <Button variant="outline" onClick={deleteSkill} disabled={isSaving}>
                  <Trash2 className="mr-2 h-4 w-4" />
                  Delete
                </Button>
              </div>
            </>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
