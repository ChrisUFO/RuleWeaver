/// Integration tests: Registry × PathResolver consistency.
///
/// For every adapter × artifact type combination, verify that:
/// - `validate_support()` returning Ok implies `path_resolver.global_path()` produces a valid absolute path.
/// - `validate_support()` returning Err implies `path_resolver.global_path()` also returns Err.
///
/// This prevents the registry and path resolver from diverging.
use tempfile::TempDir;

use ruleweaver_lib::{
    models::{
        registry::{ArtifactType, REGISTRY},
        AdapterType, Scope,
    },
    path_resolver::PathResolver,
};

fn all_adapter_types() -> Vec<AdapterType> {
    AdapterType::all()
}

fn all_artifact_types() -> Vec<ArtifactType> {
    vec![
        ArtifactType::Rule,
        ArtifactType::CommandStub,
        ArtifactType::SlashCommand,
        ArtifactType::Skill,
    ]
}

// ──────────────────────────────────────────────────────────────────────────────
// Test 1: Registry validate_support and path_resolver.global_path are consistent
// ──────────────────────────────────────────────────────────────────────────────
#[test]
fn test_path_resolver_consistent_with_registry_all_adapters() {
    let home = TempDir::new().unwrap();
    let resolver = PathResolver::new_with_home(home.path().to_path_buf(), vec![]);

    let adapters = all_adapter_types();
    let artifacts = all_artifact_types();

    for adapter in &adapters {
        for artifact in &artifacts {
            // SlashCommand intentionally requires a named path (slash_command_path()),
            // so global_path() always returns Err for it. Skip this combination.
            if *artifact == ArtifactType::SlashCommand {
                continue;
            }

            let registry_supports = REGISTRY
                .validate_support(adapter, &Scope::Global, *artifact)
                .is_ok();

            let resolver_result = resolver.global_path(*adapter, *artifact);

            match (registry_supports, resolver_result.is_ok()) {
                (true, false) => panic!(
                    "Registry says {}/{} is supported globally, but path_resolver returned Err",
                    adapter.as_str(),
                    artifact.as_str()
                ),
                (false, true) => panic!(
                    "Registry says {}/{} is NOT supported globally, but path_resolver returned Ok",
                    adapter.as_str(),
                    artifact.as_str()
                ),
                _ => {} // consistent
            }

            // If supported, the returned path must be absolute
            if registry_supports {
                let path = resolver
                    .global_path(*adapter, *artifact)
                    .unwrap()
                    .path;
                assert!(
                    path.is_absolute(),
                    "Path for {}/{} must be absolute, got: {:?}",
                    adapter.as_str(),
                    artifact.as_str(),
                    path
                );
            }
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Test 2: skill_path only works for adapters that have a skills dir configured
// ──────────────────────────────────────────────────────────────────────────────
#[test]
fn test_skill_path_only_works_when_skill_dir_configured() {
    let home = TempDir::new().unwrap();
    let resolver = PathResolver::new_with_home(home.path().to_path_buf(), vec![]);

    for adapter in all_adapter_types() {
        let entry = REGISTRY.get(&adapter).unwrap();
        let has_global_skill_dir = entry.paths.global_skills_dir.is_some();
        let skill_path_result = resolver.skill_path(adapter, "test-skill");

        match (has_global_skill_dir, skill_path_result.is_ok()) {
            (true, false) => panic!(
                "Adapter {} has global_skills_dir configured but skill_path() returned Err",
                adapter.as_str()
            ),
            (false, true) => panic!(
                "Adapter {} has no global_skills_dir but skill_path() returned Ok",
                adapter.as_str()
            ),
            _ => {} // consistent
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Test 3: Cursor has rule path (supports_rules: true) but no skill path
// ──────────────────────────────────────────────────────────────────────────────
#[test]
fn test_cursor_has_rule_path_but_no_skill_path() {
    let home = TempDir::new().unwrap();
    let resolver = PathResolver::new_with_home(home.path().to_path_buf(), vec![]);

    let rule_path = resolver.global_path(AdapterType::Cursor, ArtifactType::Rule);
    assert!(rule_path.is_ok(), "Cursor should have a valid rule path");

    let skill_path = resolver.skill_path(AdapterType::Cursor, "any-skill");
    assert!(
        skill_path.is_err(),
        "Cursor should NOT have a skill path (supports_skills: false)"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// Test 4: Windsurf has both rule path AND skill path
// ──────────────────────────────────────────────────────────────────────────────
#[test]
fn test_windsurf_has_rule_path_and_skill_path() {
    let home = TempDir::new().unwrap();
    let resolver = PathResolver::new_with_home(home.path().to_path_buf(), vec![]);

    let rule_path = resolver.global_path(AdapterType::Windsurf, ArtifactType::Rule);
    assert!(
        rule_path.is_ok(),
        "Windsurf should have a valid rule path: {:?}",
        rule_path
    );

    let skill_path = resolver.skill_path(AdapterType::Windsurf, "a-skill");
    assert!(
        skill_path.is_ok(),
        "Windsurf should have a valid skill path (supports_skills: true, dir configured): {:?}",
        skill_path
    );

    // Verify it's in the windsurf directory
    let skill_path_str = skill_path.unwrap().path.to_string_lossy().to_string();
    assert!(
        skill_path_str.contains("windsurf"),
        "Windsurf skill path must contain 'windsurf': {}",
        skill_path_str
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// Test 5: Kilo Code has rule path but no skill path (paths not configured)
// ──────────────────────────────────────────────────────────────────────────────
#[test]
fn test_kilo_code_has_rule_path_but_no_skill_path() {
    let home = TempDir::new().unwrap();
    let resolver = PathResolver::new_with_home(home.path().to_path_buf(), vec![]);

    let rule_path = resolver.global_path(AdapterType::Kilo, ArtifactType::Rule);
    assert!(rule_path.is_ok(), "Kilo Code should have a valid rule path");

    let skill_path = resolver.skill_path(AdapterType::Kilo, "a-skill");
    assert!(
        skill_path.is_err(),
        "Kilo Code should NOT have a skill path (paths not configured)"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// Test 6: All adapters produce absolute paths for supported artifact types
// ──────────────────────────────────────────────────────────────────────────────
#[test]
fn test_all_supported_paths_are_absolute() {
    let home = TempDir::new().unwrap();
    let resolver = PathResolver::new_with_home(home.path().to_path_buf(), vec![]);

    for adapter in all_adapter_types() {
        // Check rule path
        if REGISTRY
            .validate_support(&adapter, &Scope::Global, ArtifactType::Rule)
            .is_ok()
        {
            let path = resolver
                .global_path(adapter, ArtifactType::Rule)
                .unwrap()
                .path;
            assert!(
                path.is_absolute(),
                "Rule path for {} must be absolute: {:?}",
                adapter.as_str(),
                path
            );
        }

        // Check skill path for skill-supporting adapters with dirs configured
        let entry = REGISTRY.get(&adapter).unwrap();
        if entry.capabilities.supports_skills && entry.paths.global_skills_dir.is_some() {
            let path = resolver
                .skill_path(adapter, "test-skill")
                .unwrap()
                .path;
            assert!(
                path.is_absolute(),
                "Skill path for {} must be absolute: {:?}",
                adapter.as_str(),
                path
            );
        }
    }
}
