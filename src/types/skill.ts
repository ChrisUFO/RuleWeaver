export enum SkillParameterType {
  String = "String",
  Number = "Number",
  Boolean = "Boolean",
  Enum = "Enum",
  Array = "Array",
  Object = "Object",
}

export interface SkillParameter {
  name: string;
  description: string;
  param_type: SkillParameterType | string;
  required: boolean;
  default_value?: string | null;
  enum_values?: string[] | null;
}

export interface Skill {
  id: string;
  name: string;
  description: string;
  instructions: string;
  input_schema: SkillParameter[];
  directory_path: string;
  entry_point: string;
  enabled: boolean;
  created_at: number;
  updated_at: number;
}

export interface CreateSkillInput {
  name: string;
  description: string;
  instructions: string;
  input_schema: SkillParameter[];
  directory_path?: string;
  entry_point?: string;
  enabled?: boolean;
}

export interface UpdateSkillInput {
  name?: string;
  description?: string;
  instructions?: string;
  input_schema?: SkillParameter[];
  directory_path?: string;
  entry_point?: string;
  enabled?: boolean;
}

export interface TemplateFile {
  filename: string;
  content: string;
}

export interface TemplateSkill {
  template_id: string;
  metadata: CreateSkillInput;
  files: TemplateFile[];
}
