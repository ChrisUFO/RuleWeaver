#[cfg(any(test, feature = "test-helpers"))]
use std::collections::HashMap;
use std::sync::OnceLock;

#[cfg(any(test, feature = "test-helpers"))]
use std::sync::Arc;

#[cfg(not(any(test, feature = "test-helpers")))]
use crate::error::AppError;
use crate::error::Result;

#[cfg(not(any(test, feature = "test-helpers")))]
const SECRET_SERVICE_NAME: &str = "com.ruleweaver.desktop.secrets";

#[derive(Clone, Debug)]
pub struct SecretStorage {
    backend: SecretStorageBackend,
}

#[derive(Clone, Debug)]
enum SecretStorageBackend {
    #[cfg(not(any(test, feature = "test-helpers")))]
    Native,
    #[cfg(any(test, feature = "test-helpers"))]
    Memory(Arc<parking_lot::Mutex<HashMap<String, String>>>),
}

impl SecretStorage {
    pub fn global() -> Self {
        static INSTANCE: OnceLock<SecretStorage> = OnceLock::new();
        INSTANCE
            .get_or_init(|| Self {
                backend: {
                    #[cfg(any(test, feature = "test-helpers"))]
                    {
                        SecretStorageBackend::Memory(Arc::new(parking_lot::Mutex::new(
                            HashMap::new(),
                        )))
                    }
                    #[cfg(not(any(test, feature = "test-helpers")))]
                    {
                        SecretStorageBackend::Native
                    }
                },
            })
            .clone()
    }

    pub fn backend_name(&self) -> &'static str {
        match &self.backend {
            #[cfg(not(any(test, feature = "test-helpers")))]
            SecretStorageBackend::Native => "os-keychain",
            #[cfg(any(test, feature = "test-helpers"))]
            SecretStorageBackend::Memory(_) => "in-memory-test-store",
        }
    }

    pub async fn set_secret(&self, storage_key: &str, value: &str) -> Result<()> {
        match &self.backend {
            #[cfg(not(any(test, feature = "test-helpers")))]
            SecretStorageBackend::Native => {
                native_set_secret(storage_key.to_string(), value.to_string()).await
            }
            #[cfg(any(test, feature = "test-helpers"))]
            SecretStorageBackend::Memory(store) => {
                store
                    .lock()
                    .insert(storage_key.to_string(), value.to_string());
                Ok(())
            }
        }
    }

    pub async fn get_secret(&self, storage_key: &str) -> Result<Option<String>> {
        match &self.backend {
            #[cfg(not(any(test, feature = "test-helpers")))]
            SecretStorageBackend::Native => native_get_secret(storage_key.to_string()).await,
            #[cfg(any(test, feature = "test-helpers"))]
            SecretStorageBackend::Memory(store) => Ok(store.lock().get(storage_key).cloned()),
        }
    }

    pub async fn delete_secret(&self, storage_key: &str) -> Result<()> {
        match &self.backend {
            #[cfg(not(any(test, feature = "test-helpers")))]
            SecretStorageBackend::Native => native_delete_secret(storage_key.to_string()).await,
            #[cfg(any(test, feature = "test-helpers"))]
            SecretStorageBackend::Memory(store) => {
                store.lock().remove(storage_key);
                Ok(())
            }
        }
    }
}

#[cfg(any(test, feature = "test-helpers"))]
pub fn reset_test_store() {
    let storage = SecretStorage::global();
    let SecretStorageBackend::Memory(store) = storage.backend;
    store.lock().clear();
}

#[cfg(not(any(test, feature = "test-helpers")))]
async fn native_set_secret(storage_key: String, value: String) -> Result<()> {
    tokio::task::spawn_blocking(move || {
        let entry =
            keyring::Entry::new(SECRET_SERVICE_NAME, &storage_key).map_err(map_keyring_error)?;
        entry.set_password(&value).map_err(map_keyring_error)
    })
    .await
    .map_err(|err| AppError::SecureStorage {
        message: format!("Secure storage task failed: {err}"),
    })?
}

#[cfg(not(any(test, feature = "test-helpers")))]
async fn native_get_secret(storage_key: String) -> Result<Option<String>> {
    tokio::task::spawn_blocking(move || {
        let entry =
            keyring::Entry::new(SECRET_SERVICE_NAME, &storage_key).map_err(map_keyring_error)?;
        match entry.get_password() {
            Ok(value) => Ok(Some(value)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(err) => Err(map_keyring_error(err)),
        }
    })
    .await
    .map_err(|err| AppError::SecureStorage {
        message: format!("Secure storage task failed: {err}"),
    })?
}

#[cfg(not(any(test, feature = "test-helpers")))]
async fn native_delete_secret(storage_key: String) -> Result<()> {
    tokio::task::spawn_blocking(move || {
        let entry =
            keyring::Entry::new(SECRET_SERVICE_NAME, &storage_key).map_err(map_keyring_error)?;
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(err) => Err(map_keyring_error(err)),
        }
    })
    .await
    .map_err(|err| AppError::SecureStorage {
        message: format!("Secure storage task failed: {err}"),
    })?
}

#[cfg(not(any(test, feature = "test-helpers")))]
fn map_keyring_error(err: keyring::Error) -> AppError {
    AppError::SecureStorage {
        message: err.to_string(),
    }
}
