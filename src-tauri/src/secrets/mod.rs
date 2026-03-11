use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::constants::skills::SKILL_SECRET_PREFIX;
use crate::database::Database;
use crate::error::{AppError, Result};
use crate::models::{
    Command, DeleteScopedSecretInput, EffectiveSecret, ResolveScopedSecretsInput, ScopedSecret,
    SecretScope, Skill, UpsertScopedSecretInput,
};
use crate::path_resolver::PathResolver;

fn normalize_path_for_compare(path: &Path) -> String {
    let normalized = path.to_string_lossy().replace('\\', "/");
    let trimmed = normalized.trim_end_matches('/').to_string();
    if cfg!(windows) {
        trimmed.to_lowercase()
    } else {
        trimmed
    }
}

fn normalize_secret_key(key: &str) -> Result<String> {
    let trimmed = key.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation(
            "Secret key cannot be empty".to_string(),
        ));
    }
    Ok(trimmed.to_string())
}

pub fn secret_env_var_name(key: &str) -> String {
    let env_name = key
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect::<String>();

    let env_name = env_name.trim_matches('_').to_string();
    if env_name.is_empty() {
        "RULEWEAVER_SECRET".to_string()
    } else {
        env_name
    }
}

pub fn normalize_workspace_path(path: &str) -> Result<String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation(
            "Workspace path is required for this secret scope".to_string(),
        ));
    }

    let resolver = PathResolver::new()?;
    let canonical = resolver.canonicalize(&PathBuf::from(trimmed))?;
    Ok(normalize_path_for_compare(&canonical))
}

fn validate_scope_fields(
    scope: &SecretScope,
    workspace_path: &Option<String>,
    artifact_id: &Option<String>,
) -> Result<()> {
    match scope {
        SecretScope::Global => {
            if workspace_path.is_some() || artifact_id.is_some() {
                return Err(AppError::Validation(
                    "Global secrets cannot include workspace or artifact context".to_string(),
                ));
            }
        }
        SecretScope::Workspace => {
            if workspace_path.is_none() {
                return Err(AppError::Validation(
                    "Workspace secrets require a workspace path".to_string(),
                ));
            }
            if artifact_id.is_some() {
                return Err(AppError::Validation(
                    "Workspace secrets cannot include an artifact id".to_string(),
                ));
            }
        }
        SecretScope::Command | SecretScope::Skill => {
            if artifact_id.as_deref().unwrap_or_default().trim().is_empty() {
                return Err(AppError::Validation(
                    "Artifact-scoped secrets require an artifact id".to_string(),
                ));
            }
        }
    }
    Ok(())
}

fn normalize_upsert_input(mut input: UpsertScopedSecretInput) -> Result<UpsertScopedSecretInput> {
    input.key = normalize_secret_key(&input.key)?;
    if let Some(path) = input.workspace_path.as_deref() {
        input.workspace_path = Some(normalize_workspace_path(path)?);
    }
    if let Some(artifact_id) = input.artifact_id.as_mut() {
        *artifact_id = artifact_id.trim().to_string();
    }
    validate_scope_fields(&input.scope, &input.workspace_path, &input.artifact_id)?;
    Ok(input)
}

fn normalize_delete_input(mut input: DeleteScopedSecretInput) -> Result<DeleteScopedSecretInput> {
    input.key = normalize_secret_key(&input.key)?;
    if let Some(path) = input.workspace_path.as_deref() {
        input.workspace_path = Some(normalize_workspace_path(path)?);
    }
    if let Some(artifact_id) = input.artifact_id.as_mut() {
        *artifact_id = artifact_id.trim().to_string();
    }
    validate_scope_fields(&input.scope, &input.workspace_path, &input.artifact_id)?;
    Ok(input)
}

fn normalize_resolve_input(
    mut input: ResolveScopedSecretsInput,
) -> Result<ResolveScopedSecretsInput> {
    if let Some(path) = input.workspace_path.as_deref() {
        input.workspace_path = Some(normalize_workspace_path(path)?);
    }
    if let Some(artifact_id) = input.artifact_id.as_mut() {
        *artifact_id = artifact_id.trim().to_string();
    }
    if matches!(
        input.artifact_scope,
        Some(SecretScope::Global | SecretScope::Workspace)
    ) {
        return Err(AppError::Validation(
            "artifactScope must be 'command' or 'skill' when provided".to_string(),
        ));
    }
    if input.artifact_scope.is_some() && input.artifact_id.as_deref().unwrap_or_default().is_empty()
    {
        return Err(AppError::Validation(
            "artifactId is required when artifactScope is provided".to_string(),
        ));
    }
    Ok(input)
}

fn matches_resolution_context(secret: &ScopedSecret, input: &ResolveScopedSecretsInput) -> bool {
    match secret.scope {
        SecretScope::Global => true,
        SecretScope::Workspace => secret.workspace_path == input.workspace_path,
        SecretScope::Command | SecretScope::Skill => {
            let scope_match = input
                .artifact_scope
                .as_ref()
                .map(|scope| scope == &secret.scope)
                .unwrap_or(false);
            if !scope_match || secret.artifact_id != input.artifact_id {
                return false;
            }

            match (&secret.workspace_path, &input.workspace_path) {
                (Some(secret_path), Some(request_path)) => secret_path == request_path,
                (Some(_), None) => false,
                (None, _) => true,
            }
        }
    }
}

fn scope_precedence(scope: &SecretScope) -> u8 {
    match scope {
        SecretScope::Global => 0,
        SecretScope::Workspace => 1,
        SecretScope::Command | SecretScope::Skill => 2,
    }
}

fn merge_effective_secrets(secrets: Vec<ScopedSecret>) -> Vec<EffectiveSecret> {
    let mut sorted = secrets;
    sorted.sort_by(|left, right| {
        scope_precedence(&left.scope)
            .cmp(&scope_precedence(&right.scope))
            .then_with(|| left.key.to_lowercase().cmp(&right.key.to_lowercase()))
    });

    let mut effective_by_key: HashMap<String, EffectiveSecret> = HashMap::new();
    for secret in sorted {
        effective_by_key.insert(
            secret.key.to_lowercase(),
            EffectiveSecret {
                key: secret.key,
                value: secret.value,
                source_scope: secret.scope,
                workspace_path: secret.workspace_path,
                artifact_id: secret.artifact_id,
            },
        );
    }

    let mut effective = effective_by_key.into_values().collect::<Vec<_>>();
    effective.sort_by(|left, right| left.key.to_lowercase().cmp(&right.key.to_lowercase()));
    effective
}

pub fn infer_command_workspace(command: &Command) -> Result<Option<String>> {
    if let Some(base_path) = command.base_path.as_deref() {
        return normalize_workspace_path(base_path).map(Some);
    }
    match command.target_paths.as_slice() {
        [only] => normalize_workspace_path(only).map(Some),
        _ => Ok(None),
    }
}

pub fn infer_skill_workspace(skill: &Skill) -> Result<Option<String>> {
    if let Some(base_path) = skill.base_path.as_deref() {
        return normalize_workspace_path(base_path).map(Some);
    }
    match skill.target_paths.as_slice() {
        [only] => normalize_workspace_path(only).map(Some),
        _ => Ok(None),
    }
}

pub async fn list_scoped_secrets(db: &Database) -> Result<Vec<ScopedSecret>> {
    db.list_scoped_secrets().await
}

pub async fn upsert_scoped_secret(
    db: &Database,
    input: UpsertScopedSecretInput,
) -> Result<ScopedSecret> {
    db.upsert_scoped_secret(normalize_upsert_input(input)?)
        .await
}

pub async fn delete_scoped_secret(db: &Database, input: DeleteScopedSecretInput) -> Result<()> {
    db.delete_scoped_secret(normalize_delete_input(input)?)
        .await
}

pub async fn resolve_scoped_secrets(
    db: &Database,
    input: ResolveScopedSecretsInput,
) -> Result<Vec<EffectiveSecret>> {
    let normalized = normalize_resolve_input(input)?;
    let all = db.list_scoped_secrets().await?;
    Ok(merge_effective_secrets(
        all.into_iter()
            .filter(|secret| matches_resolution_context(secret, &normalized))
            .collect(),
    ))
}

pub async fn resolve_command_secret_envs(
    db: &Database,
    command: &Command,
) -> Result<Vec<(String, String)>> {
    let effective = resolve_scoped_secrets(
        db,
        ResolveScopedSecretsInput {
            workspace_path: infer_command_workspace(command)?,
            artifact_scope: Some(SecretScope::Command),
            artifact_id: Some(command.id.clone()),
        },
    )
    .await?;

    Ok(effective
        .into_iter()
        .map(|secret| (secret_env_var_name(&secret.key), secret.value))
        .collect())
}

pub async fn resolve_skill_secret_envs(
    db: &Database,
    skill: &Skill,
    allowed_keys: &HashSet<String>,
) -> Result<Vec<(String, String)>> {
    let effective = resolve_scoped_secrets(
        db,
        ResolveScopedSecretsInput {
            workspace_path: infer_skill_workspace(skill)?,
            artifact_scope: Some(SecretScope::Skill),
            artifact_id: Some(skill.id.clone()),
        },
    )
    .await?;

    let mut envs = Vec::new();
    let mut seen = HashSet::new();
    for secret in effective {
        if !allowed_keys.is_empty() && !allowed_keys.contains(&secret.key.to_lowercase()) {
            continue;
        }
        let env_name = secret_env_var_name(&secret.key);
        if seen.insert(env_name.clone()) {
            envs.push((env_name.clone(), secret.value.clone()));
            envs.push((format!("{}{}", SKILL_SECRET_PREFIX, env_name), secret.value));
        }
    }

    if allowed_keys.is_empty() {
        return Ok(envs);
    }

    let settings = db.get_all_settings().await?;
    for (key, value) in settings {
        if !allowed_keys.contains(&key.to_lowercase()) {
            continue;
        }
        let env_name = secret_env_var_name(&key);
        if seen.insert(env_name.clone()) {
            envs.push((env_name.clone(), value.clone()));
            envs.push((format!("{}{}", SKILL_SECRET_PREFIX, env_name), value));
        }
    }

    Ok(envs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CreateCommandInput, CreateSkillInput, Scope};

    #[tokio::test]
    async fn resolves_secret_precedence_global_workspace_artifact() {
        let db = Database::new_in_memory().await.unwrap();

        upsert_scoped_secret(
            &db,
            UpsertScopedSecretInput {
                key: "PROJECT_API_KEY".into(),
                value: "global".into(),
                scope: SecretScope::Global,
                workspace_path: None,
                artifact_id: None,
            },
        )
        .await
        .unwrap();
        upsert_scoped_secret(
            &db,
            UpsertScopedSecretInput {
                key: "PROJECT_API_KEY".into(),
                value: "repo-a".into(),
                scope: SecretScope::Workspace,
                workspace_path: Some("C:/repo-a".into()),
                artifact_id: None,
            },
        )
        .await
        .unwrap();
        upsert_scoped_secret(
            &db,
            UpsertScopedSecretInput {
                key: "PROJECT_API_KEY".into(),
                value: "cmd-override".into(),
                scope: SecretScope::Command,
                workspace_path: Some("C:/repo-a".into()),
                artifact_id: Some("cmd-1".into()),
            },
        )
        .await
        .unwrap();
        upsert_scoped_secret(
            &db,
            UpsertScopedSecretInput {
                key: "SHARED_TOKEN".into(),
                value: "shared".into(),
                scope: SecretScope::Global,
                workspace_path: None,
                artifact_id: None,
            },
        )
        .await
        .unwrap();

        let effective = resolve_scoped_secrets(
            &db,
            ResolveScopedSecretsInput {
                workspace_path: Some("C:/repo-a".into()),
                artifact_scope: Some(SecretScope::Command),
                artifact_id: Some("cmd-1".into()),
            },
        )
        .await
        .unwrap();

        let project = effective
            .iter()
            .find(|secret| secret.key == "PROJECT_API_KEY")
            .unwrap();
        assert_eq!(project.value, "cmd-override");
        assert_eq!(project.source_scope, SecretScope::Command);

        let shared = effective
            .iter()
            .find(|secret| secret.key == "SHARED_TOKEN")
            .unwrap();
        assert_eq!(shared.value, "shared");
        assert_eq!(shared.source_scope, SecretScope::Global);
    }

    #[tokio::test]
    async fn command_secret_envs_use_workspace_override() {
        let db = Database::new_in_memory().await.unwrap();
        let command = db
            .create_command(CreateCommandInput {
                id: Some("cmd-1".into()),
                name: "Deploy".into(),
                description: "Deploy app".into(),
                script: "echo %PROJECT_API_KEY%".into(),
                arguments: vec![],
                expose_via_mcp: true,
                is_placeholder: false,
                generate_slash_commands: false,
                slash_command_adapters: vec![],
                target_paths: vec!["C:/repo-a".into()],
                base_path: Some("C:/repo-a".into()),
                timeout_ms: None,
                max_retries: None,
            })
            .await
            .unwrap();

        upsert_scoped_secret(
            &db,
            UpsertScopedSecretInput {
                key: "PROJECT_API_KEY".into(),
                value: "global".into(),
                scope: SecretScope::Global,
                workspace_path: None,
                artifact_id: None,
            },
        )
        .await
        .unwrap();
        upsert_scoped_secret(
            &db,
            UpsertScopedSecretInput {
                key: "PROJECT_API_KEY".into(),
                value: "repo-a".into(),
                scope: SecretScope::Workspace,
                workspace_path: Some("C:/repo-a".into()),
                artifact_id: None,
            },
        )
        .await
        .unwrap();

        let envs = resolve_command_secret_envs(&db, &command).await.unwrap();
        assert!(envs
            .iter()
            .any(|(key, value)| key == "PROJECT_API_KEY" && value == "repo-a"));
    }

    #[tokio::test]
    async fn skill_secret_envs_support_scoped_and_legacy_values() {
        let db = Database::new_in_memory().await.unwrap();
        let skill = db
            .create_skill(CreateSkillInput {
                id: Some("skill-1".into()),
                name: "Build Skill".into(),
                description: "Skill".into(),
                instructions: "Run steps".into(),
                scope: Scope::Local,
                input_schema: vec![],
                directory_path: "./skills/build".into(),
                entry_point: "run.cmd".into(),
                enabled: true,
                target_adapters: vec![],
                target_paths: vec!["C:/repo-a".into()],
                base_path: Some("C:/repo-a".into()),
            })
            .await
            .unwrap();

        upsert_scoped_secret(
            &db,
            UpsertScopedSecretInput {
                key: "PROJECT_API_KEY".into(),
                value: "repo-a".into(),
                scope: SecretScope::Workspace,
                workspace_path: Some("C:/repo-a".into()),
                artifact_id: None,
            },
        )
        .await
        .unwrap();
        db.set_setting("LEGACY_TOKEN", "legacy-value")
            .await
            .unwrap();

        let allowed = HashSet::from(["project_api_key".to_string(), "legacy_token".to_string()]);
        let envs = resolve_skill_secret_envs(&db, &skill, &allowed)
            .await
            .unwrap();

        assert!(envs
            .iter()
            .any(|(key, value)| key == "PROJECT_API_KEY" && value == "repo-a"));
        assert!(envs
            .iter()
            .any(|(key, value)| key == "SKILL_SECRET_PROJECT_API_KEY" && value == "repo-a"));
        assert!(envs
            .iter()
            .any(|(key, value)| key == "LEGACY_TOKEN" && value == "legacy-value"));
    }
}
