/// Integration tests: Rule lifecycle across create → sync/reconcile → update → delete.
///
/// These tests use an in-memory SQLite DB and a PathResolver pointed at a TempDir
/// so no real filesystem paths are contaminated.
mod common;

use std::sync::Arc;
use tempfile::TempDir;

use ruleweaver_lib::{
    database::Database,
    models::{AdapterType, CreateRuleInput, Scope, UpdateRuleInput},
    path_resolver::PathResolver,
    reconciliation::ReconciliationEngine,
};

/// Build an isolated test environment: in-memory DB + PathResolver in a TempDir.
async fn make_env() -> (Arc<Database>, TempDir) {
    let db = Arc::new(Database::new_in_memory().await.unwrap());
    let dir = TempDir::new().unwrap();
    (db, dir)
}

// ──────────────────────────────────────────────────────────────────────────────
// Test 1: Create rule → desired state contains expected paths
// ──────────────────────────────────────────────────────────────────────────────
#[tokio::test]
async fn test_rule_create_produces_desired_state() {
    let (db, home_dir) = make_env().await;

    db.create_rule(CreateRuleInput {
        id: None,
        name: "test-rule".into(),
        description: "A test rule".into(),
        content: "Always use TypeScript.".into(),
        scope: Scope::Global,
        target_paths: None,
        enabled_adapters: vec![AdapterType::ClaudeCode, AdapterType::OpenCode],
        enabled: true,
    })
    .await
    .unwrap();

    let engine = common::make_engine(db, home_dir.path());
    let desired = engine.compute_desired_state().await.unwrap();

    // At least two paths should be in desired state (one per adapter)
    assert!(
        desired.expected_paths.len() >= 2,
        "Expected at least 2 paths for ClaudeCode + OpenCode, got {}",
        desired.expected_paths.len()
    );

    // All expected paths should reference the rule content
    for (_path, artifact) in &desired.expected_paths {
        if artifact.artifact_type == ruleweaver_lib::models::registry::ArtifactType::Rule {
            assert_eq!(artifact.name, "test-rule");
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Test 2: Create rule → reconcile → verify files written
// ──────────────────────────────────────────────────────────────────────────────
#[tokio::test]
async fn test_rule_create_reconcile_writes_files() {
    let (db, home_dir) = make_env().await;

    db.create_rule(CreateRuleInput {
        id: None,
        name: "code-standards".into(),
        description: "Coding standards".into(),
        content: "Use Rust for all backends.".into(),
        scope: Scope::Global,
        target_paths: None,
        enabled_adapters: vec![AdapterType::ClaudeCode],
        enabled: true,
    })
    .await
    .unwrap();

    let engine = common::make_engine(db, home_dir.path());
    let result = engine.reconcile(false, None).await.unwrap();

    assert!(result.success, "Reconcile should succeed");
    assert!(result.created >= 1, "At least one file should be created");

    // Verify the actual file exists under our temp home
    let expected_path = home_dir.path().join(".claude").join("CLAUDE.md");
    assert!(
        expected_path.exists(),
        "Expected CLAUDE.md at {:?}",
        expected_path
    );

    let content = std::fs::read_to_string(&expected_path).unwrap();
    assert!(
        content.contains("Use Rust for all backends."),
        "CLAUDE.md should contain the rule content"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// Test 3: Update rule content → reconcile → file content updated
// ──────────────────────────────────────────────────────────────────────────────
#[tokio::test]
async fn test_rule_update_reconcile_updates_file() {
    let (db, home_dir) = make_env().await;

    let rule = db
        .create_rule(CreateRuleInput {
            id: None,
            name: "style-guide".into(),
            description: "Style guidelines".into(),
            content: "Original content.".into(),
            scope: Scope::Global,
            target_paths: None,
            enabled_adapters: vec![AdapterType::ClaudeCode],
            enabled: true,
        })
        .await
        .unwrap();

    let engine = common::make_engine(db.clone(), home_dir.path());
    engine.reconcile(false, None).await.unwrap();

    // Update the content
    db.update_rule(
        &rule.id,
        UpdateRuleInput {
            content: Some("Updated content after change.".into()),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let engine2 = common::make_engine(db, home_dir.path());
    let result = engine2.reconcile(false, None).await.unwrap();

    assert!(result.success);
    assert!(result.updated >= 1, "At least one file should be updated");

    let expected_path = home_dir.path().join(".claude").join("CLAUDE.md");
    let content = std::fs::read_to_string(&expected_path).unwrap();
    assert!(
        content.contains("Updated content after change."),
        "File should contain updated content"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// Test 4: Delete rule → reconcile → orphan file removed
// ──────────────────────────────────────────────────────────────────────────────
#[tokio::test]
async fn test_rule_delete_reconcile_removes_orphan() {
    let (db, home_dir) = make_env().await;

    let rule = db
        .create_rule(CreateRuleInput {
            id: None,
            name: "ephemeral-rule".into(),
            description: "Will be deleted".into(),
            content: "This rule will be removed.".into(),
            scope: Scope::Global,
            target_paths: None,
            enabled_adapters: vec![AdapterType::ClaudeCode],
            enabled: true,
        })
        .await
        .unwrap();

    let engine = common::make_engine(db.clone(), home_dir.path());
    engine.reconcile(false, None).await.unwrap();

    // Confirm file exists
    let file_path = home_dir.path().join(".claude").join("CLAUDE.md");
    assert!(
        file_path.exists(),
        "File should exist after initial reconcile"
    );

    // Delete the rule
    db.delete_rule(&rule.id).await.unwrap();

    let engine2 = common::make_engine(db, home_dir.path());
    let result = engine2.reconcile(false, None).await.unwrap();

    assert!(result.success);
    // The orphaned file should be removed
    assert!(
        result.removed >= 1,
        "Orphaned file should be removed, got: {:?}",
        result
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// Test 5: Global rule writes to global path only (not local)
// ──────────────────────────────────────────────────────────────────────────────
#[tokio::test]
async fn test_global_rule_writes_to_global_path_only() {
    let (db, home_dir) = make_env().await;
    let repo_root = TempDir::new().unwrap();

    db.create_rule(CreateRuleInput {
        id: None,
        name: "global-only".into(),
        description: "Only global scope".into(),
        content: "Global rule content.".into(),
        scope: Scope::Global,
        target_paths: None,
        enabled_adapters: vec![AdapterType::ClaudeCode],
        enabled: true,
    })
    .await
    .unwrap();

    let resolver = PathResolver::new_with_home(
        home_dir.path().to_path_buf(),
        vec![repo_root.path().to_path_buf()],
    );
    let engine = ReconciliationEngine::new_with_resolver(db, resolver);
    let desired = engine.compute_desired_state().await.unwrap();

    // All rule paths for this global rule should be under home_dir, not repo_root
    for (path, artifact) in &desired.expected_paths {
        if artifact.artifact_type == ruleweaver_lib::models::registry::ArtifactType::Rule {
            assert!(
                path.starts_with(home_dir.path().to_string_lossy().as_ref()),
                "Global rule path should be under home_dir, got: {}",
                path
            );
        }
    }
}
