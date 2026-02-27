#![allow(dead_code)]
//! Shared test helpers for integration tests.
//!
//! Include in each test file with:
//!   mod common;
//! then call `common::make_db().await` and `common::make_engine(db, home)`.

use std::sync::Arc;

use ruleweaver_lib::{
    database::Database,
    path_resolver::PathResolver,
    reconciliation::ReconciliationEngine,
};

/// Create an isolated in-memory database for integration tests.
pub async fn make_db() -> Arc<Database> {
    Arc::new(Database::new_in_memory().await.unwrap())
}

/// Create a `ReconciliationEngine` pointed at `home` with an empty repo-roots list.
pub fn make_engine(db: Arc<Database>, home: &std::path::Path) -> ReconciliationEngine {
    let resolver = PathResolver::new_with_home(home.to_path_buf(), vec![]);
    ReconciliationEngine::new_with_resolver(db, resolver)
}
