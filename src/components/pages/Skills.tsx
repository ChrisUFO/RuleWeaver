import { useEffect, useMemo, useState } from "react";
import { Plus, Trash2, FolderOpen } from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import { Badge } from "@/components/ui/badge";
import { api } from "@/lib/tauri";
import { useToast } from "@/components/ui/toast";
import type { Skill, SkillParameter } from "@/types/skill";
import { Scope } from "@/types/rule";
import { SkillSchemaEditor } from "@/components/skills/SkillSchemaEditor";
import { TemplateBrowser } from "@/components/skills/TemplateBrowser";
import { Select } from "@/components/ui/select";
import { useRepositoryRoots } from "@/hooks/useRepositoryRoots";

export function Skills() {
  const [skills, setSkills] = useState<Skill[]>([]);
  const [selectedId, setSelectedId] = useState("");
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [instructions, setInstructions] = useState("");
  const [inputSchema, setInputSchema] = useState<SkillParameter[]>([]);
  const [entryPoint, setEntryPoint] = useState("");
  const [scope, setScope] = useState<Scope>("global");
  const [directoryPath, setDirectoryPath] = useState("");
  const [enabled, setEnabled] = useState(true);
  const { roots: availableRepos } = useRepositoryRoots();
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
    setScope(selected.scope);
    setDirectoryPath(selected.directory_path || "");
    setEnabled(selected.enabled);
  }, [selected]);

  const createSkill = async () => {
    setIsSaving(true);
    try {
      const created = await api.skills.create({
        name: "New Skill",
        description: "Describe this workflow",
        instructions: "Step 1\nStep 2",
        scope: "global",
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
        scope,
        input_schema: inputSchema,
        directory_path: scope === "local" ? directoryPath : undefined,
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
      <Card className="glass-card premium-shadow border-none overflow-hidden">
        <CardHeader className="space-y-4 bg-white/5 pb-6">
          <div className="flex items-center justify-between">
            <CardTitle className="text-sm font-semibold tracking-wide uppercase text-muted-foreground/80">
              Skills
            </CardTitle>
            <div className="flex gap-2">
              <TemplateBrowser onInstalled={loadSkills} />
              <Button
                size="sm"
                onClick={createSkill}
                disabled={isSaving}
                className="glow-primary h-8"
              >
                <Plus className="mr-1.5 h-3.5 w-3.5" />
                New
              </Button>
            </div>
          </div>
          <CardDescription className="text-xs">Complex multi-step workflows</CardDescription>
        </CardHeader>
        <CardContent className="space-y-1.5 pt-4 px-2">
          {skills.map((skill) => (
            <button
              key={skill.id}
              className={cn(
                "w-full group relative overflow-hidden flex flex-col items-start rounded-xl px-4 py-3 text-left transition-all duration-300",
                selectedId === skill.id
                  ? "bg-primary/10 border border-primary/20 premium-shadow"
                  : "hover:bg-white/5 border border-transparent hover:border-white/5"
              )}
              onClick={() => setSelectedId(skill.id)}
            >
              <div className="flex w-full items-center justify-between gap-2">
                <div
                  className={cn(
                    "truncate font-semibold text-sm transition-colors",
                    selectedId === skill.id
                      ? "text-primary"
                      : "text-foreground group-hover:text-primary/80"
                  )}
                >
                  {skill.name}
                </div>
                {!skill.enabled && (
                  <Badge
                    variant="secondary"
                    className="h-4 text-[9px] px-1.5 uppercase font-bold tracking-tighter"
                  >
                    Disabled
                  </Badge>
                )}
              </div>
              <div className="mt-1 truncate text-[11px] text-muted-foreground/60 group-hover:text-muted-foreground/80 opacity-80">
                {skill.description}
              </div>
            </button>
          ))}
          {skills.length === 0 && (
            <p className="text-xs text-muted-foreground/60 text-center py-8">
              No skills installed.
            </p>
          )}
        </CardContent>
      </Card>

      <Card className="glass-card premium-shadow border-none overflow-hidden">
        <CardHeader className="bg-white/5 pb-4">
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="text-sm font-semibold tracking-wide uppercase text-primary/80">
                {selected ? name : "Select a Skill"}
              </CardTitle>
              <CardDescription>Define reusable instructions and workflow context.</CardDescription>
            </div>
            {selected && selected.directory_path && (
              <Button
                variant="outline"
                size="sm"
                onClick={openFolder}
                className="glass border-white/5 hover:bg-white/5"
              >
                <FolderOpen className="mr-2 h-4 w-4" /> Open Folder
              </Button>
            )}
          </div>
        </CardHeader>
        <CardContent className="space-y-6 pt-6">
          <div className="rounded-xl border border-amber-500/20 bg-amber-500/5 p-4 text-[11px] text-amber-200/60 leading-relaxed">
            <span className="font-bold text-amber-500 uppercase tracking-widest mr-2">
              Warning:
            </span>
            Skills execute shell commands with your current user privileges. Treat imported or
            shared skills as trusted code only.
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
                {scope === "local" && (
                  <div className="space-y-2 md:col-span-2">
                    <label className="text-sm font-medium">Directory Path (for local skill)</label>
                    {availableRepos.length > 0 && (
                      <Select
                        value={availableRepos.includes(directoryPath) ? directoryPath : ""}
                        onChange={(value) => {
                          if (value) setDirectoryPath(value);
                        }}
                        options={[
                          { value: "", label: "Select configured repository" },
                          ...availableRepos.map((repo) => ({ value: repo, label: repo })),
                        ]}
                        aria-label="Select local repository"
                      />
                    )}
                    <Input
                      value={directoryPath}
                      onChange={(e) => setDirectoryPath(e.target.value)}
                      placeholder="/absolute/path/to/project/.agent/skills/my-skill"
                    />
                    {availableRepos.length === 0 && (
                      <p className="text-xs text-muted-foreground">
                        No configured repositories found. Add them in Settings.
                      </p>
                    )}
                  </div>
                )}
              </div>

              <div className="space-y-2">
                <label className="text-xs font-bold uppercase tracking-widest text-muted-foreground/60">
                  Instructions (SKILL.md)
                </label>
                <textarea
                  value={instructions}
                  onChange={(e) => setInstructions(e.target.value)}
                  className="min-h-60 w-full rounded-xl border border-white/5 bg-black/40 p-4 text-[13px] font-mono shadow-inner focus:outline-none focus:ring-1 focus:ring-primary/40 leading-relaxed text-primary/90 selection:bg-primary/20"
                  placeholder="Write detailed workflow instructions for the AI"
                />
              </div>

              <SkillSchemaEditor schema={inputSchema} onChange={setInputSchema} />

              <div className="grid gap-4 md:grid-cols-2">
                <div className="space-y-2">
                  <label className="text-sm font-medium">Scope</label>
                  <div className="flex gap-2">
                    <Button
                      type="button"
                      variant={scope === "global" ? "default" : "outline"}
                      size="sm"
                      onClick={() => setScope("global")}
                      className="flex-1"
                    >
                      Global
                    </Button>
                    <Button
                      type="button"
                      variant={scope === "local" ? "default" : "outline"}
                      size="sm"
                      onClick={() => setScope("local")}
                      className="flex-1"
                    >
                      Local
                    </Button>
                  </div>
                </div>
              </div>

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
