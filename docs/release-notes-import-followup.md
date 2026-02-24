# Release Notes: Import Follow-up (Issues #25 and #12)

## Highlights

- Added repository root management in Settings so local repositories are configured once and reused across artifacts.
- Expanded AI tool import discovery to include additional legacy and alternate tool path layouts.
- Added command-level `target_paths` support end-to-end (model, DB, validation, UI, and sync outputs).
- Improved import UX with source filtering in history, conflict retry action, and additional edge-case handling.

## Hardening

- URL imports validate scheme/host safety and re-validate final redirected URL.
- Directory and clipboard imports enforce limits and validation constraints.
- Command target paths must fall within configured repository roots.

## Verification

- `npm run typecheck`
- `npm run test`
- `npm run test:coverage`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`
