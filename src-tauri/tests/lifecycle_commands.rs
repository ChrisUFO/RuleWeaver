/// Integration tests: Command lifecycle across create → stub sync → reconcile → delete.
mod common;

use tempfile::TempDir;

use ruleweaver_lib::models::{AdapterType, CreateCommandInput};

// ──────────────────────────────────────────────────────────────────────────────
// Test 1: Create command → reconcile → command stub file written
// ──────────────────────────────────────────────────────────────────────────────
#[tokio::test]
async fn test_command_create_reconcile_writes_stub() {
    let db = common::make_db().await;
    let home_dir = TempDir::new().unwrap();

    db.create_command(CreateCommandInput {
        id: None,
        name: "format-code".into(),
        description: "Format the codebase".into(),
        script: "npm run format".into(),
        arguments: vec![],
        expose_via_mcp: true,
        is_placeholder: false,
        generate_slash_commands: false,
        slash_command_adapters: vec![],
        target_paths: vec![],
        base_path: None,
        timeout_ms: None,
        max_retries: None,
    })
    .await
    .unwrap();

    let engine = common::make_engine(db, home_dir.path());
    let desired = engine.compute_desired_state().await.unwrap();

    // Should have command stub paths for supported adapters
    let stub_paths: Vec<&String> = desired
        .expected_paths
        .iter()
        .filter(|(_, a)| {
            a.artifact_type == ruleweaver_lib::models::registry::ArtifactType::CommandStub
        })
        .map(|(p, _)| p)
        .collect();

    assert!(
        !stub_paths.is_empty(),
        "Command stub paths should be present in desired state"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// Test 2: Delete command → reconcile → stub files removed
// ──────────────────────────────────────────────────────────────────────────────
#[tokio::test]
async fn test_command_delete_reconcile_removes_stubs() {
    let db = common::make_db().await;
    let home_dir = TempDir::new().unwrap();

    let cmd = db
        .create_command(CreateCommandInput {
            id: None,
            name: "deploy".into(),
            description: "Deploy the app".into(),
            script: "./deploy.sh".into(),
            arguments: vec![],
            expose_via_mcp: true,
            is_placeholder: false,
            generate_slash_commands: false,
            slash_command_adapters: vec![],
            target_paths: vec![],
            base_path: None,
            timeout_ms: None,
            max_retries: None,
        })
        .await
        .unwrap();
    // Initial reconcile to write stubs
    let engine = common::make_engine(db.clone(), home_dir.path());
    let create_result = engine.reconcile(false, None).await.unwrap();
    assert!(create_result.success);
    let initial_created = create_result.created;
    assert!(initial_created >= 1, "Should create stub files");

    // Delete the command
    db.delete_command(&cmd.id).await.unwrap();

    // Reconcile should remove orphaned stubs
    let engine2 = common::make_engine(db, home_dir.path());
    let delete_result = engine2.reconcile(false, None).await.unwrap();

    assert!(delete_result.success);
    assert!(
        delete_result.removed >= 1,
        "Orphaned command stub files should be removed"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// Test 3: Cursor does NOT get command stub file (supports_command_stubs: false)
// ──────────────────────────────────────────────────────────────────────────────
#[tokio::test]
async fn test_cursor_gets_no_command_stub() {
    let db = common::make_db().await;
    let home_dir = TempDir::new().unwrap();

    // Create a command (commands go to all supported adapters by default)
    db.create_command(CreateCommandInput {
        id: None,
        name: "test-cmd".into(),
        description: "Test command".into(),
        script: "echo hello".into(),
        arguments: vec![],
        expose_via_mcp: false,
        is_placeholder: false,
        generate_slash_commands: false,
        slash_command_adapters: vec![],
        target_paths: vec![],
        base_path: None,
        timeout_ms: None,
        max_retries: None,
    })
    .await
    .unwrap();

    let engine = common::make_engine(db, home_dir.path());
    let desired = engine.compute_desired_state().await.unwrap();

    // Verify no command stub paths reference the Cursor adapter
    let cursor_stub_paths: Vec<&String> = desired
        .expected_paths
        .iter()
        .filter(|(_, a)| {
            a.artifact_type == ruleweaver_lib::models::registry::ArtifactType::CommandStub
                && a.adapter == AdapterType::Cursor
        })
        .map(|(p, _)| p)
        .collect();

    assert!(
        cursor_stub_paths.is_empty(),
        "Cursor should receive no command stub files (supports_command_stubs: false), got: {:?}",
        cursor_stub_paths
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// Test 4: Slash command create → desired state contains slash command paths
// ──────────────────────────────────────────────────────────────────────────────
#[tokio::test]
async fn test_slash_command_create_produces_desired_paths() {
    let db = common::make_db().await;
    let home_dir = TempDir::new().unwrap();

    // Command with slash commands enabled for ClaudeCode
    db.create_command(CreateCommandInput {
        id: None,
        name: "review".into(),
        description: "Review code".into(),
        script: "echo review".into(),
        arguments: vec![],
        expose_via_mcp: false,
        is_placeholder: false,
        generate_slash_commands: true,
        slash_command_adapters: vec!["claude-code".into()],
        target_paths: vec![],
        base_path: None,
        timeout_ms: None,
        max_retries: None,
    })
    .await
    .unwrap();

    let engine = common::make_engine(db, home_dir.path());
    let desired = engine.compute_desired_state().await.unwrap();

    let slash_paths: Vec<&String> = desired
        .expected_paths
        .iter()
        .filter(|(_, a)| {
            a.artifact_type == ruleweaver_lib::models::registry::ArtifactType::SlashCommand
        })
        .map(|(p, _)| p)
        .collect();

    assert!(
        !slash_paths.is_empty(),
        "Slash command paths should be in desired state"
    );

    // Should be a .md file in .claude/commands/
    assert!(
        slash_paths
            .iter()
            .any(|p| p.contains("claude") && p.ends_with(".md")),
        "Slash command for Claude Code should be a .md file: {:?}",
        slash_paths
    );
}
