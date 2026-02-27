/// Integration tests: Skill lifecycle across create → reconcile → adapter targeting → delete.
use std::sync::Arc;
use tempfile::TempDir;

use ruleweaver_lib::{
    database::Database,
    models::{CreateSkillInput, Scope, UpdateSkillInput},
    path_resolver::PathResolver,
    reconciliation::ReconciliationEngine,
};

async fn make_db() -> Arc<Database> {
    Arc::new(Database::new_in_memory().await.unwrap())
}

fn make_engine(db: Arc<Database>, home: &std::path::Path) -> ReconciliationEngine {
    let resolver = PathResolver::new_with_home(home.to_path_buf(), vec![]);
    ReconciliationEngine::new_with_resolver(db, resolver)
}

// ──────────────────────────────────────────────────────────────────────────────
// Test 1: Create skill → reconcile → SKILL.md written to supported adapter dir
// ──────────────────────────────────────────────────────────────────────────────
#[tokio::test]
async fn test_skill_create_reconcile_writes_skill_md() {
    let db = make_db().await;
    let home_dir = TempDir::new().unwrap();

    db.create_skill(CreateSkillInput {
        id: None,
        name: "my-skill".into(),
        description: "A test skill".into(),
        instructions: "Do the thing step by step.".into(),
        scope: Scope::Global,
        input_schema: vec![],
        directory_path: "".into(),
        entry_point: "".into(),
        enabled: true,
        target_adapters: vec![],  // all supported
        target_paths: vec![],
    })
    .await
    .unwrap();

    let engine = make_engine(db, home_dir.path());
    let result = engine.reconcile(false, None).await.unwrap();

    assert!(result.success, "Reconcile must succeed");
    assert!(result.created >= 1, "Should create at least one SKILL.md");

    // ClaudeCode should have a SKILL.md
    let skill_path = home_dir
        .path()
        .join(".claude")
        .join("skills")
        .join("my-skill")
        .join("SKILL.md");
    assert!(
        skill_path.exists(),
        "SKILL.md should be written to Claude Code skill dir: {:?}",
        skill_path
    );

    let content = std::fs::read_to_string(&skill_path).unwrap();
    assert!(
        content.contains("Do the thing step by step."),
        "SKILL.md should contain the skill instructions"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// Test 2: Skill with adapter targeting → only targeted adapter gets SKILL.md
// ──────────────────────────────────────────────────────────────────────────────
#[tokio::test]
async fn test_skill_adapter_targeting_limits_distribution() {
    let db = make_db().await;
    let home_dir = TempDir::new().unwrap();

    // Target only claude-code
    db.create_skill(CreateSkillInput {
        id: None,
        name: "targeted-skill".into(),
        description: "Only for Claude".into(),
        instructions: "Claude-only instructions.".into(),
        scope: Scope::Global,
        input_schema: vec![],
        directory_path: "".into(),
        entry_point: "".into(),
        enabled: true,
        target_adapters: vec!["claude-code".into()],
        target_paths: vec![],
    })
    .await
    .unwrap();

    let engine = make_engine(db, home_dir.path());
    let desired = engine.compute_desired_state().await.unwrap();

    // Collect all skill paths
    let skill_paths: Vec<&String> = desired
        .expected_paths
        .iter()
        .filter(|(_, a)| {
            a.artifact_type == ruleweaver_lib::models::registry::ArtifactType::Skill
        })
        .map(|(p, _)| p)
        .collect();

    // Only 1 skill path (Claude Code) should be present
    assert_eq!(
        skill_paths.len(),
        1,
        "Only ClaudeCode should receive the targeted skill, got {} paths: {:?}",
        skill_paths.len(),
        skill_paths
    );

    // The one path must be under .claude/skills
    assert!(
        skill_paths[0].contains("claude") || skill_paths[0].contains(".claude"),
        "Skill path must be in Claude Code's skills dir: {}",
        skill_paths[0]
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// Test 3: Delete skill → reconcile → SKILL.md files removed
// ──────────────────────────────────────────────────────────────────────────────
#[tokio::test]
async fn test_skill_delete_reconcile_removes_skill_md() {
    let db = make_db().await;
    let home_dir = TempDir::new().unwrap();

    let skill = db
        .create_skill(CreateSkillInput {
            id: None,
            name: "delete-me".into(),
            description: "Will be deleted".into(),
            instructions: "Instructions.".into(),
            scope: Scope::Global,
            input_schema: vec![],
            directory_path: "".into(),
            entry_point: "".into(),
            enabled: true,
            target_adapters: vec!["claude-code".into()],
            target_paths: vec![],
        })
        .await
        .unwrap();

    // Create initial files
    let engine = make_engine(db.clone(), home_dir.path());
    engine.reconcile(false, None).await.unwrap();

    let skill_path = home_dir
        .path()
        .join(".claude")
        .join("skills")
        .join("delete-me")
        .join("SKILL.md");
    assert!(skill_path.exists(), "SKILL.md should exist after first reconcile");

    // Delete the skill
    db.delete_skill(&skill.id).await.unwrap();

    let engine2 = make_engine(db, home_dir.path());
    let result = engine2.reconcile(false, None).await.unwrap();

    assert!(result.success);
    assert!(result.removed >= 1, "Orphaned SKILL.md should be removed");
}

// ──────────────────────────────────────────────────────────────────────────────
// Test 4: Update skill instructions → reconcile → SKILL.md content updated
// ──────────────────────────────────────────────────────────────────────────────
#[tokio::test]
async fn test_skill_update_reconcile_updates_content() {
    let db = make_db().await;
    let home_dir = TempDir::new().unwrap();

    let skill = db
        .create_skill(CreateSkillInput {
            id: None,
            name: "updatable-skill".into(),
            description: "Test update".into(),
            instructions: "Original instructions.".into(),
            scope: Scope::Global,
            input_schema: vec![],
            directory_path: "".into(),
            entry_point: "".into(),
            enabled: true,
            target_adapters: vec!["claude-code".into()],
            target_paths: vec![],
        })
        .await
        .unwrap();

    let engine = make_engine(db.clone(), home_dir.path());
    engine.reconcile(false, None).await.unwrap();

    // Update instructions
    db.update_skill(
        &skill.id,
        UpdateSkillInput {
            instructions: Some("Updated instructions with new steps.".into()),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let engine2 = make_engine(db, home_dir.path());
    let result = engine2.reconcile(false, None).await.unwrap();

    assert!(result.success);
    assert!(result.updated >= 1, "SKILL.md should be updated");

    let skill_path = home_dir
        .path()
        .join(".claude")
        .join("skills")
        .join("updatable-skill")
        .join("SKILL.md");
    let content = std::fs::read_to_string(&skill_path).unwrap();
    assert!(
        content.contains("Updated instructions with new steps."),
        "SKILL.md must contain updated instructions"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// Test 5: Cursor does NOT get SKILL.md (supports_skills: false)
// ──────────────────────────────────────────────────────────────────────────────
#[tokio::test]
async fn test_cursor_gets_no_skill_files() {
    let db = make_db().await;
    let home_dir = TempDir::new().unwrap();

    // Create skill targeting cursor — should be silently skipped
    db.create_skill(CreateSkillInput {
        id: None,
        name: "cursor-skill".into(),
        description: "Attempted cursor skill".into(),
        instructions: "Instructions for cursor.".into(),
        scope: Scope::Global,
        input_schema: vec![],
        directory_path: "".into(),
        entry_point: "".into(),
        enabled: true,
        target_adapters: vec!["cursor".into()],
        target_paths: vec![],
    })
    .await
    .unwrap();

    let engine = make_engine(db, home_dir.path());
    let desired = engine.compute_desired_state().await.unwrap();

    // No skill paths should be in desired state (cursor is silently skipped)
    let skill_paths: Vec<&String> = desired
        .expected_paths
        .iter()
        .filter(|(_, a)| {
            a.artifact_type == ruleweaver_lib::models::registry::ArtifactType::Skill
        })
        .map(|(p, _)| p)
        .collect();

    assert!(
        skill_paths.is_empty(),
        "Cursor skill targeting should produce no skill paths; got: {:?}",
        skill_paths
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// Test 6: Windsurf DOES get SKILL.md (supports_skills: true, paths configured)
// ──────────────────────────────────────────────────────────────────────────────
#[tokio::test]
async fn test_windsurf_gets_skill_files() {
    let db = make_db().await;
    let home_dir = TempDir::new().unwrap();

    db.create_skill(CreateSkillInput {
        id: None,
        name: "windsurf-skill".into(),
        description: "Windsurf skill".into(),
        instructions: "Instructions for windsurf.".into(),
        scope: Scope::Global,
        input_schema: vec![],
        directory_path: "".into(),
        entry_point: "".into(),
        enabled: true,
        target_adapters: vec!["windsurf".into()],
        target_paths: vec![],
    })
    .await
    .unwrap();

    let engine = make_engine(db, home_dir.path());
    let desired = engine.compute_desired_state().await.unwrap();

    let skill_paths: Vec<&String> = desired
        .expected_paths
        .iter()
        .filter(|(_, a)| {
            a.artifact_type == ruleweaver_lib::models::registry::ArtifactType::Skill
        })
        .map(|(p, _)| p)
        .collect();

    assert!(
        !skill_paths.is_empty(),
        "Windsurf should receive skill files but got no paths"
    );

    // Verify the path contains windsurf
    assert!(
        skill_paths
            .iter()
            .any(|p| p.contains("windsurf")),
        "Skill path should be under windsurf directory: {:?}",
        skill_paths
    );
}
