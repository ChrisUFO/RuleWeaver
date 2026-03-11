use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::constants::skills::SKILL_SECRET_PREFIX;
use crate::database::{Database, ScopedSecretRecord};
use crate::error::{AppError, Result};
use crate::models::{
    Command, DeleteScopedSecretInput, EffectiveSecret, ResolveScopedSecretsInput, ScopedSecret,
    SecretScope, SecretStorageStatus, Skill, UpsertScopedSecretInput,
};
use crate::path_resolver::PathResolver;
use crate::secure_storage::SecretStorage;
use sha2::{Digest, Sha256};

const MASKED_SECRET_VALUE: &str = "••••••••";

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

fn validate_secret_key_for_write(key: &str) -> Result<()> {
    let mut chars = key.chars();
    let Some(first) = chars.next() else {
        return Err(AppError::Validation(
            "Secret key cannot be empty".to_string(),
        ));
    };

    if !(first.is_ascii_alphabetic() || first == '_')
        || !chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    {
        return Err(AppError::Validation(
            "Secret key must match environment variable naming rules (letters, numbers, underscores, and it cannot start with a number)".to_string(),
        ));
    }

    Ok(())
}

fn normalize_secret_value_for_write(value: &str) -> Result<String> {
    if value.trim().is_empty() {
        return Err(AppError::Validation(
            "Secret value cannot be empty".to_string(),
        ));
    }
    Ok(value.to_string())
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
    validate_secret_key_for_write(&input.key)?;
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

fn mask_scoped_secret(mut secret: ScopedSecret) -> ScopedSecret {
    secret.value = MASKED_SECRET_VALUE.to_string();
    secret
}

fn scoped_secret_storage_key(
    namespace: &str,
    scope: &SecretScope,
    key: &str,
    workspace_path: Option<&str>,
    artifact_id: Option<&str>,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(namespace.as_bytes());
    hasher.update(b"|");
    hasher.update(scope.as_str().as_bytes());
    hasher.update(b"|");
    hasher.update(key.to_lowercase().as_bytes());
    hasher.update(b"|");
    hasher.update(workspace_path.unwrap_or_default().as_bytes());
    hasher.update(b"|");
    hasher.update(artifact_id.unwrap_or_default().as_bytes());
    format!("scoped-secret-{:x}", hasher.finalize())
}

fn scoped_secret_storage_key_for_record(namespace: &str, secret: &ScopedSecretRecord) -> String {
    scoped_secret_storage_key(
        namespace,
        &secret.scope,
        &secret.key,
        secret.workspace_path.as_deref(),
        secret.artifact_id.as_deref(),
    )
}

async fn persist_secret_value(
    storage: &SecretStorage,
    namespace: &str,
    input: &UpsertScopedSecretInput,
) -> Result<()> {
    let storage_key = scoped_secret_storage_key(
        namespace,
        &input.scope,
        &input.key,
        input.workspace_path.as_deref(),
        input.artifact_id.as_deref(),
    );
    storage.set_secret(&storage_key, &input.value).await
}

async fn migrate_plaintext_scoped_secrets(db: &Database, storage: &SecretStorage) -> Result<()> {
    let namespace = db.secret_namespace();
    for secret in db.list_scoped_secret_records().await? {
        if secret.value.is_empty() {
            continue;
        }

        let storage_key = scoped_secret_storage_key_for_record(namespace, &secret);
        storage.set_secret(&storage_key, &secret.value).await?;
        db.clear_scoped_secret_plaintext_value(&secret.id).await?;
    }

    Ok(())
}

async fn migrate_legacy_allowlisted_settings(db: &Database, storage: &SecretStorage) -> Result<()> {
    let namespace = db.secret_namespace();
    let Some(allowlist_raw) = db.get_setting("mcp_secrets_allowlist").await? else {
        return Ok(());
    };

    for key in allowlist_raw
        .split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
    {
        let Some(value) = db.get_setting(key).await? else {
            continue;
        };

        if value.trim().is_empty() {
            db.delete_setting(key).await?;
            continue;
        }

        let input = normalize_upsert_input(UpsertScopedSecretInput {
            key: key.to_string(),
            value,
            scope: SecretScope::Global,
            workspace_path: None,
            artifact_id: None,
        })?;
        persist_secret_value(storage, namespace, &input).await?;
        db.upsert_scoped_secret(input).await?;
        db.delete_setting(key).await?;
    }

    Ok(())
}

async fn ensure_secure_secret_storage(db: &Database) -> Result<SecretStorage> {
    let storage = SecretStorage::global();
    migrate_plaintext_scoped_secrets(db, &storage).await?;
    migrate_legacy_allowlisted_settings(db, &storage).await?;
    Ok(storage)
}

async fn resolve_secret_record(
    storage: &SecretStorage,
    namespace: &str,
    secret: ScopedSecretRecord,
) -> Result<ScopedSecret> {
    let storage_key = scoped_secret_storage_key_for_record(namespace, &secret);
    let value = storage
        .get_secret(&storage_key)
        .await?
        .ok_or_else(|| AppError::SecureStorage {
            message: format!(
                "Secure secret '{}' could not be resolved from {}",
                secret.key,
                storage.backend_name()
            ),
        })?;

    Ok(ScopedSecret {
        id: secret.id,
        key: secret.key,
        value,
        scope: secret.scope,
        workspace_path: secret.workspace_path,
        artifact_id: secret.artifact_id,
        created_at: secret.created_at,
        updated_at: secret.updated_at,
    })
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

fn matches_resolution_context(
    secret: &ScopedSecretRecord,
    input: &ResolveScopedSecretsInput,
) -> bool {
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
    ensure_secure_secret_storage(db).await?;
    Ok(db
        .list_scoped_secrets()
        .await?
        .into_iter()
        .map(mask_scoped_secret)
        .collect())
}

pub fn get_secret_storage_status() -> SecretStorageStatus {
    let storage = SecretStorage::global();
    let backend = storage.backend_name().to_string();
    SecretStorageStatus {
        stores_secrets_in_os_credential_manager: backend != "in-memory-test-store",
        backend,
        exports_include_secrets: false,
        imports_include_secrets: false,
    }
}

pub async fn upsert_scoped_secret(
    db: &Database,
    input: UpsertScopedSecretInput,
) -> Result<ScopedSecret> {
    let mut normalized = normalize_upsert_input(input)?;
    normalized.value = normalize_secret_value_for_write(&normalized.value)?;
    let storage = ensure_secure_secret_storage(db).await?;
    persist_secret_value(&storage, db.secret_namespace(), &normalized).await?;
    Ok(mask_scoped_secret(
        db.upsert_scoped_secret(normalized).await?,
    ))
}

pub async fn delete_scoped_secret(db: &Database, input: DeleteScopedSecretInput) -> Result<()> {
    let normalized = normalize_delete_input(input)?;
    let storage = ensure_secure_secret_storage(db).await?;
    let storage_key = scoped_secret_storage_key(
        db.secret_namespace(),
        &normalized.scope,
        &normalized.key,
        normalized.workspace_path.as_deref(),
        normalized.artifact_id.as_deref(),
    );
    db.delete_scoped_secret(normalized).await?;
    storage.delete_secret(&storage_key).await
}

pub async fn resolve_scoped_secrets(
    db: &Database,
    input: ResolveScopedSecretsInput,
) -> Result<Vec<EffectiveSecret>> {
    let normalized = normalize_resolve_input(input)?;
    let storage = ensure_secure_secret_storage(db).await?;
    let namespace = db.secret_namespace();
    let mut all = Vec::new();
    for secret in db.list_scoped_secret_records().await? {
        if matches_resolution_context(&secret, &normalized) {
            all.push(resolve_secret_record(&storage, namespace, secret).await?);
        }
    }
    Ok(merge_effective_secrets(all))
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
    if allowed_keys.is_empty() {
        return Ok(Vec::new());
    }

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
    async fn skill_secret_envs_migrate_legacy_allowlisted_settings() {
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
        db.set_setting("mcp_secrets_allowlist", "PROJECT_API_KEY,LEGACY_TOKEN")
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
        assert_eq!(db.get_setting("LEGACY_TOKEN").await.unwrap(), None);
    }

    #[tokio::test]
    async fn skill_secret_envs_require_explicit_allowlist() {
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

        let envs = resolve_skill_secret_envs(&db, &skill, &HashSet::new())
            .await
            .unwrap();

        assert!(envs.is_empty());
    }

    #[tokio::test]
    async fn secret_list_and_upsert_mask_values() {
        let db = Database::new_in_memory().await.unwrap();

        let saved = upsert_scoped_secret(
            &db,
            UpsertScopedSecretInput {
                key: "PROJECT_API_KEY".into(),
                value: "super-secret".into(),
                scope: SecretScope::Global,
                workspace_path: None,
                artifact_id: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(saved.value, MASKED_SECRET_VALUE);

        let listed = list_scoped_secrets(&db).await.unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].value, MASKED_SECRET_VALUE);

        let stored = db.list_scoped_secret_records().await.unwrap();
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].value, "");

        let resolved = resolve_scoped_secrets(&db, ResolveScopedSecretsInput::default())
            .await
            .unwrap();
        assert_eq!(resolved[0].value, "super-secret");
    }

    #[tokio::test]
    async fn resolves_existing_plaintext_secret_after_secure_storage_migration() {
        let db = Database::new_in_memory().await.unwrap();

        db.upsert_scoped_secret(UpsertScopedSecretInput {
            key: "LEGACY_SECRET".into(),
            value: "legacy-plaintext".into(),
            scope: SecretScope::Global,
            workspace_path: None,
            artifact_id: None,
        })
        .await
        .unwrap();
        db.force_set_scoped_secret_plaintext_value("LEGACY_SECRET", "legacy-plaintext")
            .await
            .unwrap();

        let resolved = resolve_scoped_secrets(&db, ResolveScopedSecretsInput::default())
            .await
            .unwrap();

        assert!(resolved
            .iter()
            .any(|secret| secret.key == "LEGACY_SECRET" && secret.value == "legacy-plaintext"));
        assert_eq!(db.list_scoped_secret_records().await.unwrap()[0].value, "");
    }

    #[test]
    fn secret_storage_status_reports_no_secret_export_or_import() {
        let status = get_secret_storage_status();
        assert!(!status.exports_include_secrets);
        assert!(!status.imports_include_secrets);
    }

    #[test]
    fn upsert_secret_rejects_invalid_env_var_names() {
        let invalid = normalize_upsert_input(UpsertScopedSecretInput {
            key: "PROJECT=API_KEY".into(),
            value: "secret".into(),
            scope: SecretScope::Global,
            workspace_path: None,
            artifact_id: None,
        });

        assert!(invalid.is_err());

        let leading_digit = normalize_upsert_input(UpsertScopedSecretInput {
            key: "1PROJECT_API_KEY".into(),
            value: "secret".into(),
            scope: SecretScope::Global,
            workspace_path: None,
            artifact_id: None,
        });

        assert!(leading_digit.is_err());
    }
}
