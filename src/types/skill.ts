import { Scope } from "./rule";

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
  paramType: SkillParameterType | string;
  required: boolean;
  defaultValue?: string | null;
  enumValues?: string[] | null;
}

export interface Skill {
  id: string;
  name: string;
  description: string;
  instructions: string;
  scope: Scope;
  inputSchema: SkillParameter[];
  directoryPath: string;
  entryPoint: string;
  enabled: boolean;
  createdAt: number;
  updatedAt: number;
}

export interface CreateSkillInput {
  id?: string;
  name: string;
  description: string;
  instructions: string;
  scope: Scope;
  inputSchema: SkillParameter[];
  directoryPath?: string;
  entryPoint?: string;
  enabled?: boolean;
}

export interface UpdateSkillInput {
  name?: string;
  description?: string;
  instructions?: string;
  scope?: Scope;
  inputSchema?: SkillParameter[];
  directoryPath?: string;
  entryPoint?: string;
  enabled?: boolean;
}

export interface TemplateFile {
  filename: string;
  content: string;
}

export interface TemplateSkill {
  templateId: string;
  theme: string;
  metadata: CreateSkillInput;
  files: TemplateFile[];
}
