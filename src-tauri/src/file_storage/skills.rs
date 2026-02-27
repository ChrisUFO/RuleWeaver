use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::database::Database;
use crate::error::{AppError, Result};
use crate::models::{CreateSkillInput, Scope, Skill, SkillParameter, UpdateSkillInput};

use crate::constants::{SKILLS_DIR_NAME, SKILL_INSTRUCTIONS_FILE, SKILL_METADATA_FILE};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SkillMetadata {
    pub id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub entry_point: String,
    #[serde(default)]
    pub input_schema: Vec<SkillParameter>,
    pub scope: Scope,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

fn default_true() -> bool {
    true
}

pub fn get_global_skills_dir() -> Result<PathBuf> {
    let base = dirs::data_local_dir()
        .or_else(dirs::data_dir)
        .ok_or_else(|| AppError::Path("Could not determine data directory".to_string()))?;
    Ok(base.join("RuleWeaver").join(SKILLS_DIR_NAME))
}

pub fn load_skills_from_disk() -> Result<Vec<Skill>> {
    let global_dir = get_global_skills_dir()?;
    let mut skills = Vec::new();

    if !global_dir.exists() {
        return Ok(skills);
    }

    for entry in WalkDir::new(&global_dir).min_depth(1).max_depth(1) {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();
        if path.is_dir() {
            if let Ok(skill) = load_skill_from_directory(path) {
                skills.push(skill);
            }
        }
    }

    Ok(skills)
}

pub fn load_skill_from_directory(dir: &Path) -> Result<Skill> {
    let metadata_path = dir.join(SKILL_METADATA_FILE);
    let instructions_path = dir.join(SKILL_INSTRUCTIONS_FILE);

    if !metadata_path.exists() {
        return Err(AppError::InvalidInput {
            message: format!("Missing {} in skill directory", SKILL_METADATA_FILE),
        });
    }

    let metadata_content = fs::read_to_string(&metadata_path)?;
    let metadata: SkillMetadata =
        serde_json::from_str(&metadata_content).map_err(|e| AppError::InvalidInput {
            message: format!("Failed to parse {}: {}", SKILL_METADATA_FILE, e),
        })?;

    let instructions = if instructions_path.exists() {
        fs::read_to_string(&instructions_path)?
    } else {
        String::new()
    };

    let entry_point_path = dir.join(&metadata.entry_point);
    if !entry_point_path.exists() {
        return Err(AppError::InvalidInput {
            message: format!("Entry point '{}' does not exist", metadata.entry_point),
        });
    }

    let now = Utc::now();
    let id = metadata
        .id
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    // Convert created_at/updated_at or generate new
    let created_at = metadata
        .created_at
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or(now);

    let updated_at = metadata
        .updated_at
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or(now);

    Ok(Skill {
        id,
        name: metadata.name,
        description: metadata.description.unwrap_or_default(),
        instructions,
        scope: metadata.scope,
        input_schema: metadata.input_schema,
        enabled: metadata.enabled,
        directory_path: dir.to_string_lossy().to_string(),
        entry_point: metadata.entry_point,
        target_adapters: Vec::new(),
        target_paths: Vec::new(),
        base_path: None,
        created_at,
        updated_at,
    })
}

pub fn validate_skill_directory_path(path: &Path) -> Result<()> {
    // 1. Ensure it's absolute
    if !path.is_absolute() {
        return Err(AppError::InvalidInput {
            message: "Skill directory path must be absolute".to_string(),
        });
    }

    // 2. Security: Check for path traversal components
    for component in path.components() {
        if let std::path::Component::ParentDir = component {
            return Err(AppError::InvalidInput {
                message: "Path traversal sequences (..) are not allowed".to_string(),
            });
        }
    }

    // 3. Ensure it's within user's home directory for safety
    let home = dirs::home_dir()
        .ok_or_else(|| AppError::Path("Could not determine home directory".to_string()))?;
    if !path.starts_with(&home) {
        return Err(AppError::InvalidInput {
            message: "Skill directory must be within your home directory".to_string(),
        });
    }

    Ok(())
}

pub fn save_skill_to_disk(skill: &Skill) -> Result<PathBuf> {
    let skill_dir = if !skill.directory_path.is_empty() {
        PathBuf::from(&skill.directory_path)
    } else {
        let base_dir = match skill.scope {
            Scope::Global => get_global_skills_dir()?,
            Scope::Local => {
                let path = PathBuf::from(&skill.directory_path);
                validate_skill_directory_path(&path)?;
                path
            }
        };

        if !base_dir.exists() {
            fs::create_dir_all(&base_dir)?;
        }

        // Sanitize name for directory
        let safe_name = skill
            .name
            .to_lowercase()
            .replace(|c: char| !c.is_alphanumeric(), "-")
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-");

        let mut dir = base_dir.join(&safe_name);

        // Handle collisions
        let mut counter = 1;
        while dir.exists() {
            dir = base_dir.join(format!("{}-{}", safe_name, counter));
            counter += 1;
        }
        dir
    };

    if !skill_dir.exists() {
        fs::create_dir_all(&skill_dir)?;
    } else if skill.id.is_empty() {
        // This is a new skill creation with an existing directory
        return Err(AppError::InvalidInput {
            message: format!("Directory already exists: {}", skill_dir.display()),
        });
    }

    // Write SKILL.md
    let instructions_path = skill_dir.join(SKILL_INSTRUCTIONS_FILE);
    fs::write(&instructions_path, &skill.instructions)?;

    // Write skill.json
    let metadata = SkillMetadata {
        id: Some(skill.id.clone()),
        name: skill.name.clone(),
        description: Some(skill.description.clone()),
        entry_point: skill.entry_point.clone(),
        input_schema: skill.input_schema.clone(),
        scope: skill.scope,
        enabled: skill.enabled,
        created_at: Some(skill.created_at.to_rfc3339()),
        updated_at: Some(skill.updated_at.to_rfc3339()),
    };

    let metadata_path = skill_dir.join(SKILL_METADATA_FILE);
    let json_content = serde_json::to_string_pretty(&metadata)?;
    fs::write(&metadata_path, &json_content)?;

    Ok(skill_dir)
}

pub async fn sync_skills_to_db(db: &Database) -> Result<u32> {
    let skills = load_skills_from_disk()?;
    let mut count = 0;

    for skill in skills {
        // Try to update, if not found, create
        if db.get_skill_by_id(&skill.id).await.is_ok() {
            let update_input = UpdateSkillInput {
                name: Some(skill.name.clone()),
                description: Some(skill.description.clone()),
                instructions: Some(skill.instructions.clone()),
                input_schema: Some(skill.input_schema.clone()),
                scope: Some(skill.scope),
                directory_path: Some(skill.directory_path.clone()),
                entry_point: Some(skill.entry_point.clone()),
                enabled: Some(skill.enabled),
                ..Default::default()
            };
            db.update_skill(&skill.id, update_input).await?;
        } else {
            let create_input = CreateSkillInput {
                id: Some(skill.id.clone()),
                name: skill.name.clone(),
                description: skill.description.clone(),
                instructions: skill.instructions.clone(),
                scope: skill.scope,
                input_schema: skill.input_schema.clone(),
                directory_path: skill.directory_path.clone(),
                entry_point: skill.entry_point.clone(),
                enabled: skill.enabled,
                ..Default::default()
            };
            db.create_skill(create_input).await?;
        }
        count += 1;
    }

    Ok(count)
}

pub fn delete_skill_from_disk(skill: &Skill) -> Result<()> {
    let skill_dir = PathBuf::from(&skill.directory_path);
    if skill_dir.exists() && skill_dir.is_dir() {
        // Security: Canonicalize both paths to prevent directory traversal / symlink bypasses
        let canonical_skill_dir = std::fs::canonicalize(&skill_dir).map_err(AppError::Io)?;
        let global_dir = get_global_skills_dir()?;
        let canonical_global_dir = if global_dir.exists() {
            std::fs::canonicalize(&global_dir).map_err(AppError::Io)?
        } else {
            global_dir
        };

        if canonical_skill_dir.starts_with(&canonical_global_dir) {
            fs::remove_dir_all(canonical_skill_dir)?;
        }
    }
    Ok(())
}
