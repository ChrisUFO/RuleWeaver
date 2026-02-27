/// Generate `docs/SUPPORT_MATRIX.md` from the canonical tool registry.
///
/// Usage (from workspace root):
///   cargo run --manifest-path src-tauri/Cargo.toml --bin gen_docs
///
/// The generated file is committed to the repository. A CI job (`docs-check`) regenerates
/// it and fails if the result differs from the committed version, ensuring docs never drift
/// from the registry.
fn main() {
    let content = ruleweaver_lib::models::registry::generate_support_matrix();

    // Resolve output path: CARGO_MANIFEST_DIR is src-tauri/; workspace root is one level up.
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("src-tauri must have a parent workspace root");
    let output_path = workspace_root.join("docs").join("SUPPORT_MATRIX.md");

    std::fs::create_dir_all(output_path.parent().unwrap())
        .expect("Failed to create docs/ directory");
    std::fs::write(&output_path, &content).expect("Failed to write docs/SUPPORT_MATRIX.md");

    println!(
        "Generated: {}  ({} bytes)",
        output_path.display(),
        content.len()
    );
}
