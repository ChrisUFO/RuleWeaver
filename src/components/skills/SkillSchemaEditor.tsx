import { Plus, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import { SkillParameter, SkillParameterType } from "@/types/skill";

export function SkillSchemaEditor({
  schema,
  onChange,
}: {
  schema: SkillParameter[];
  onChange: (schema: SkillParameter[]) => void;
}) {
  const addParam = () => {
    onChange([
      ...schema,
      {
        name: "",
        description: "",
        param_type: SkillParameterType.String,
        required: true,
      },
    ]);
  };

  const updateParam = (index: number, updates: Partial<SkillParameter>) => {
    if (index < 0 || index >= schema.length) {
      console.error("Invalid parameter index", { index, schemaLength: schema.length });
      return;
    }
    const next = [...schema];
    next[index] = { ...next[index], ...updates };
    onChange(next);
  };

  const removeParam = (index: number) => {
    if (index < 0 || index >= schema.length) {
      console.error("Invalid parameter index for removal", { index, schemaLength: schema.length });
      return;
    }
    onChange(schema.filter((_, i) => i !== index));
  };

  return (
    <div className="space-y-4 rounded-md border p-4">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-medium">Input Schema Parameters</h3>
        <Button variant="outline" size="sm" onClick={addParam}>
          <Plus className="mr-2 h-4 w-4" /> Add Parameter
        </Button>
      </div>

      {schema.length === 0 ? (
        <p className="text-xs text-muted-foreground">
          No parameters defined. The skill will run without arguments.
        </p>
      ) : (
        <div className="space-y-4">
          {schema.map((param, i) => (
            <div key={i} className="grid gap-3 rounded-md border border-muted bg-muted/30 p-3">
              <div className="flex items-start justify-between gap-4">
                <div className="grid flex-1 gap-2 sm:grid-cols-2">
                  <div className="space-y-1">
                    <label className="text-xs font-medium">Name</label>
                    <Input
                      size={1}
                      placeholder="e.g. file_path"
                      value={param.name}
                      onChange={(e) => updateParam(i, { name: e.target.value })}
                    />
                  </div>
                  <div className="space-y-1">
                    <label className="text-xs font-medium">Type</label>
                    <select
                      className="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium file:text-foreground placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
                      value={param.param_type}
                      onChange={(e) =>
                        updateParam(i, { param_type: e.target.value as SkillParameterType })
                      }
                    >
                      {Object.values(SkillParameterType).map((t) => (
                        <option key={t} value={t}>
                          {t}
                        </option>
                      ))}
                    </select>
                  </div>
                  <div className="sm:col-span-2 space-y-1">
                    <label className="text-xs font-medium">Description</label>
                    <Input
                      placeholder="Explain what this parameter does"
                      value={param.description}
                      onChange={(e) => updateParam(i, { description: e.target.value })}
                    />
                  </div>

                  {param.param_type === SkillParameterType.Enum && (
                    <div className="sm:col-span-2 space-y-1">
                      <label className="text-xs font-medium">Enum Values (comma separated)</label>
                      <Input
                        placeholder="Option1, Option2, Option3"
                        value={param.enum_values?.join(", ") ?? ""}
                        onChange={(e) => {
                          const vals = e.target.value
                            .split(",")
                            .map((s) => s.trim())
                            .filter(Boolean);
                          updateParam(i, { enum_values: vals.length > 0 ? vals : null });
                        }}
                      />
                    </div>
                  )}

                  <div className="space-y-1">
                    <label className="text-xs font-medium">Default (optional)</label>
                    <Input
                      placeholder="Leave blank for no default"
                      value={param.default_value ?? ""}
                      onChange={(e) => updateParam(i, { default_value: e.target.value || null })}
                    />
                  </div>
                </div>

                <div className="flex flex-col items-end gap-3">
                  <Button variant="ghost" size="icon" onClick={() => removeParam(i)}>
                    <Trash2 className="h-4 w-4 text-destructive" />
                  </Button>
                  <div className="flex items-center gap-2">
                    <span className="text-xs">Required</span>
                    <Switch
                      checked={param.required}
                      onCheckedChange={(c) => updateParam(i, { required: c })}
                    />
                  </div>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
