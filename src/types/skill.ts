export interface Skill {
  id: string;
  name: string;
  description: string;
  instructions: string;
  enabled: boolean;
  created_at: number;
  updated_at: number;
}

export interface CreateSkillInput {
  name: string;
  description: string;
  instructions: string;
  enabled?: boolean;
}

export interface UpdateSkillInput {
  name?: string;
  description?: string;
  instructions?: string;
  enabled?: boolean;
}
