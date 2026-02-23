import { useEffect, useMemo, useState } from "react";
import { Plus, Trash2, FolderOpen } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import { Badge } from "@/components/ui/badge";
import { api } from "@/lib/tauri";
import { useToast } from "@/components/ui/toast";
import type { Skill, SkillParameter } from "@/types/skill";
import { SkillSchemaEditor } from "@/components/skills/SkillSchemaEditor";
import { TemplateBrowser } from "@/components/skills/TemplateBrowser";

export function Skills() {
  const [skills, setSkills] = useState<Skill[]>([]);
  const [selectedId, setSelectedId] = useState("");
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [instructions, setInstructions] = useState("");
  const [inputSchema, setInputSchema] = useState<SkillParameter[]>([]);
  const [entryPoint, setEntryPoint] = useState("");
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
      setInputSchema([]);
      setEntryPoint("");
      setEnabled(true);
      return;
    }
    setName(selected.name);
    setDescription(selected.description);
    setInstructions(selected.instructions);
    setInputSchema(selected.input_schema || []);
    setEntryPoint(selected.entry_point || "");
    setEnabled(selected.enabled);
  }, [selected]);

  const createSkill = async () => {
    setIsSaving(true);
    try {
      const created = await api.skills.create({
        name: "New Skill",
        description: "Describe this workflow",
        instructions: "Step 1\nStep 2",
        input_schema: [],
        entry_point: "run.sh",
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
        input_schema: inputSchema,
        entry_point: entryPoint,
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

  const openFolder = async () => {
    if (!selected?.directory_path) return;
    try {
      await api.app.openInExplorer(selected.directory_path);
    } catch {
      addToast({
        title: "Failed to Open",
        description: "Could not open directory",
        variant: "error",
      });
    }
  };

  return (
    <div className="grid gap-6 lg:grid-cols-[320px,1fr]">
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <CardTitle>Skills</CardTitle>
            <div className="flex gap-2">
              <TemplateBrowser onInstalled={loadSkills} />
              <Button size="sm" onClick={createSkill} disabled={isSaving}>
                <Plus className="mr-2 h-4 w-4" />
                New
              </Button>
            </div>
          </div>
          <CardDescription>Complex multi-step workflows</CardDescription>
        </CardHeader>
        <CardContent className="space-y-2">
          {skills.map((skill) => (
            <button
              key={skill.id}
              className={`w-full flex-col items-start rounded-md border px-3 py-2 text-left transition ${
                selectedId === skill.id ? "border-primary bg-accent" : "hover:bg-accent"
              }`}
              onClick={() => setSelectedId(skill.id)}
            >
              <div className="flex w-full items-center justify-between">
                <div className="font-medium truncate">{skill.name}</div>
                {!skill.enabled && <Badge variant="secondary">Disabled</Badge>}
              </div>
              <div className="text-xs text-muted-foreground truncate opacity-80">
                {skill.description}
              </div>
            </button>
          ))}
          {skills.length === 0 && (
            <p className="text-sm text-muted-foreground p-2">No skills installed.</p>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle>{selected ? "Edit Skill" : "Select a Skill"}</CardTitle>
              <CardDescription>Define reusable instructions and workflow context.</CardDescription>
            </div>
            {selected && selected.directory_path && (
              <Button variant="outline" size="sm" onClick={openFolder}>
                <FolderOpen className="mr-2 h-4 w-4" /> Open Folder
              </Button>
            )}
          </div>
        </CardHeader>
        <CardContent className="space-y-6">
          <div className="rounded-md border border-amber-300 bg-amber-50 p-3 text-xs text-amber-900">
            Security warning: Skills execute shell commands with your current user privileges. Treat
            imported or shared skills as trusted code only.
          </div>
          {!selected ? (
            <p className="text-sm text-muted-foreground">Select a skill or create a new one.</p>
          ) : (
            <>
              <div className="grid gap-4 md:grid-cols-2">
                <div className="space-y-2">
                  <label className="text-sm font-medium">Name</label>
                  <Input
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                    placeholder="Skill name"
                  />
                </div>
                <div className="space-y-2">
                  <label className="text-sm font-medium">Entry Point (e.g. run.sh, index.js)</label>
                  <Input
                    value={entryPoint}
                    onChange={(e) => setEntryPoint(e.target.value)}
                    placeholder="main.sh"
                  />
                </div>
                <div className="space-y-2 md:col-span-2">
                  <label className="text-sm font-medium">Description</label>
                  <Input
                    value={description}
                    onChange={(e) => setDescription(e.target.value)}
                    placeholder="What does this do?"
                  />
                </div>
              </div>

              <div className="space-y-2">
                <label className="text-sm font-medium">LLM Instructions (SKILL.md format)</label>
                <textarea
                  value={instructions}
                  onChange={(e) => setInstructions(e.target.value)}
                  className="min-h-48 w-full rounded-md border bg-background p-3 text-sm font-mono shadow-inner"
                  placeholder="Write detailed workflow instructions for the AI"
                />
              </div>

              <SkillSchemaEditor schema={inputSchema} onChange={setInputSchema} />

              <div className="flex items-center justify-between rounded-md border p-4 bg-muted/20">
                <div className="space-y-0.5">
                  <div className="text-sm font-medium">Enable Skill</div>
                  <div className="text-xs text-muted-foreground">
                    Allow this skill to be used by the MCP server
                  </div>
                </div>
                <Switch checked={enabled} onCheckedChange={setEnabled} />
              </div>
              <div className="flex gap-2 pt-2 border-t">
                <Button onClick={saveSkill} disabled={isSaving}>
                  {isSaving ? "Saving..." : "Save Changes"}
                </Button>
                <Button variant="outline" onClick={deleteSkill} disabled={isSaving}>
                  <Trash2 className="mr-2 h-4 w-4" />
                  Delete Skill
                </Button>
              </div>
            </>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
