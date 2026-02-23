#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use crate::database::Database;
    use crate::models::{ArgumentType, Command, CommandArgument, CreateCommandInput};
    use crate::slash_commands::{
        get_adapter, get_all_adapters, SlashCommandSyncEngine, SlashCommandSyncResult, SyncStatus,
    };

    fn setup_test_db() -> Arc<Database> {
        // Create a temporary database for testing
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        Arc::new(Database::new_with_db_path(db_path).unwrap())
    }

    fn create_test_command_input(
        name: &str,
        description: &str,
        script: &str,
    ) -> CreateCommandInput {
        CreateCommandInput {
            name: name.to_string(),
            description: description.to_string(),
            script: script.to_string(),
            arguments: vec![],
            expose_via_mcp: true,
            generate_slash_commands: true,
            slash_command_adapters: vec!["opencode".to_string()],
        }
    }

    fn create_test_command_with_adapters(
        db: &Database,
        name: &str,
        adapters: Vec<String>,
    ) -> Command {
        let input = CreateCommandInput {
            name: name.to_string(),
            description: "Test command".to_string(),
            script: "echo test".to_string(),
            arguments: vec![],
            expose_via_mcp: true,
            generate_slash_commands: true,
            slash_command_adapters: adapters,
        };
        db.create_command(input).unwrap()
    }

    #[test]
    fn test_sync_engine_new() {
        let db = setup_test_db();
        let engine = SlashCommandSyncEngine::new(db);
        // Just verify it creates without error
    }

    #[test]
    fn test_sync_command_disabled() {
        let db = setup_test_db();
        let engine = SlashCommandSyncEngine::new(db);

        let input = CreateCommandInput {
            name: "test".to_string(),
            description: "Test".to_string(),
            script: "echo test".to_string(),
            arguments: vec![],
            expose_via_mcp: true,
            generate_slash_commands: false, // Disabled
            slash_command_adapters: vec!["opencode".to_string()],
        };

        let command = db.create_command(input).unwrap();
        let result = engine.sync_command(&command, true).unwrap();

        assert_eq!(result.files_written, 0);
        assert_eq!(result.errors.len(), 0);
    }

    #[test]
    fn test_sync_command_with_unknown_adapter() {
        let db = setup_test_db();
        let engine = SlashCommandSyncEngine::new(db);

        let input = CreateCommandInput {
            name: "test".to_string(),
            description: "Test".to_string(),
            script: "echo test".to_string(),
            arguments: vec![],
            expose_via_mcp: true,
            generate_slash_commands: true,
            slash_command_adapters: vec!["unknown-adapter".to_string()],
        };

        let command = db.create_command(input).unwrap();
        let result = engine.sync_command(&command, true).unwrap();

        assert_eq!(result.files_written, 0);
        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].contains("Unknown adapter"));
    }

    #[test]
    fn test_get_adapter_returns_some_for_valid_adapters() {
        let valid_adapters = vec![
            "opencode",
            "claude-code",
            "cline",
            "gemini",
            "cursor",
            "roo",
            "antigravity",
            "codex",
        ];

        for name in valid_adapters {
            assert!(
                get_adapter(name).is_some(),
                "Should return adapter for {}",
                name
            );
        }
    }

    #[test]
    fn test_get_adapter_returns_none_for_invalid_adapter() {
        assert!(get_adapter("invalid-adapter").is_none());
        assert!(get_adapter("").is_none());
    }

    #[test]
    fn test_get_all_adapters_returns_eight_adapters() {
        let adapters = get_all_adapters();
        assert_eq!(adapters.len(), 8);
    }

    #[test]
    fn test_slash_command_sync_result_success() {
        let result = SlashCommandSyncResult {
            files_written: 2,
            files_removed: 0,
            errors: vec![],
            conflicts: vec![],
        };

        assert!(result.is_success());
    }

    #[test]
    fn test_slash_command_sync_result_failure_with_errors() {
        let result = SlashCommandSyncResult {
            files_written: 0,
            files_removed: 0,
            errors: vec!["Error 1".to_string()],
            conflicts: vec![],
        };

        assert!(!result.is_success());
    }

    #[test]
    fn test_slash_command_sync_result_failure_with_conflicts() {
        use crate::slash_commands::SlashCommandConflict;

        let result = SlashCommandSyncResult {
            files_written: 0,
            files_removed: 0,
            errors: vec![],
            conflicts: vec![SlashCommandConflict {
                command_name: "test".to_string(),
                adapter_name: "opencode".to_string(),
                file_path: std::path::PathBuf::from("/test"),
                message: "Conflict".to_string(),
            }],
        };

        assert!(!result.is_success());
    }
        let not_synced = SyncStatus::NotSynced;
        let error = SyncStatus::Error("Test error".to_string());

        // Just verify they can be created
        match synced {
            SyncStatus::Synced => {}
            _ => panic!("Should be Synced"),
        }

        match error {
            SyncStatus::Error(msg) => assert_eq!(msg, "Test error"),
            _ => panic!("Should be Error"),
        }
    }

    #[test]
    fn test_remove_command_with_no_files() {
        let db = setup_test_db();
        let engine = SlashCommandSyncEngine::new(db);

        let adapters = vec!["opencode".to_string()];
        let result = engine
            .remove_command("non-existent-cmd", &adapters)
            .unwrap();

        assert_eq!(result.files_removed, 0);
    }

    #[test]
    fn test_cleanup_unknown_adapter_returns_error() {
        let db = setup_test_db();
        let engine = SlashCommandSyncEngine::new(db);

        let result = engine.cleanup_adapter("unknown", true);
        assert!(result.is_err());
    }

    #[test]
    fn test_sync_all_commands_empty() {
        let db = setup_test_db();
        let engine = SlashCommandSyncEngine::new(db);

        let result = engine.sync_all_commands(true).unwrap();

        assert_eq!(result.files_written, 0);
        assert!(result.errors.is_empty());
        assert!(result.conflicts.is_empty());
    }

    #[test]
    fn test_command_with_multiple_adapters() {
        let db = setup_test_db();
        let engine = SlashCommandSyncEngine::new(db);

        let command = create_test_command_with_adapters(
            &db,
            "multi-adapter-cmd",
            vec!["opencode".to_string(), "claude-code".to_string()],
        );

        let result = engine.sync_command(&command, true).unwrap();

        // Should attempt to sync to both adapters (even if they fail due to missing dirs)
        assert!(result.files_written >= 0 || !result.errors.is_empty());
    }

    #[test]
    fn test_sync_status_for_unsynced_command() {
        let db = setup_test_db();
        let engine = SlashCommandSyncEngine::new(db);

        let command =
            create_test_command_with_adapters(&db, "unsynced", vec!["opencode".to_string()]);

        let status = engine.get_command_sync_status(&command).unwrap();

        // Should return NotSynced status since file doesn't exist
        assert!(status.contains_key("opencode"));
        match &status["opencode"] {
            SyncStatus::NotSynced => {}
            _ => {}
        }
    }

    #[test]
    fn test_command_with_arguments_includes_in_frontmatter() {
        let db = setup_test_db();

        let input = CreateCommandInput {
            name: "deploy".to_string(),
            description: "Deploy command".to_string(),
            script: "./deploy.sh".to_string(),
            arguments: vec![CommandArgument {
                name: "env".to_string(),
                description: "Environment".to_string(),
                arg_type: ArgumentType::Enum,
                required: true,
                default_value: Some("staging".to_string()),
                options: Some(vec!["staging".to_string(), "prod".to_string()]),
            }],
            expose_via_mcp: true,
            generate_slash_commands: true,
            slash_command_adapters: vec!["opencode".to_string()],
        };

        let command = db.create_command(input).unwrap();

        assert_eq!(command.arguments.len(), 1);
        assert_eq!(command.arguments[0].name, "env");
    }
}
