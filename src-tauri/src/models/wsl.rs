use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::models::AdapterType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum WslMode {
    #[default]
    Windows,
    Wsl,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WslAdapterConfig {
    pub mode: WslMode,
    pub distribution: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub home_dir: Option<PathBuf>,
}

impl Default for WslAdapterConfig {
    fn default() -> Self {
        Self {
            mode: WslMode::Windows,
            distribution: None,
            home_dir: None,
        }
    }
}

impl WslAdapterConfig {
    pub fn windows() -> Self {
        Self {
            mode: WslMode::Windows,
            distribution: None,
            home_dir: None,
        }
    }

    pub fn wsl(distribution: String, home_dir: PathBuf) -> Self {
        Self {
            mode: WslMode::Wsl,
            distribution: Some(distribution),
            home_dir: Some(home_dir),
        }
    }

    pub fn is_wsl(&self) -> bool {
        self.mode == WslMode::Wsl
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WslConfig {
    pub enabled: bool,
    pub default_distribution: Option<String>,
    pub adapters: HashMap<AdapterType, WslAdapterConfig>,
}

impl WslConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn with_default_distribution(mut self, distribution: String) -> Self {
        self.default_distribution = Some(distribution);
        self
    }

    pub fn with_adapter(mut self, adapter: AdapterType, config: WslAdapterConfig) -> Self {
        self.adapters.insert(adapter, config);
        self
    }

    pub fn get_adapter_config(&self, adapter: AdapterType) -> WslAdapterConfig {
        self.adapters.get(&adapter).cloned().unwrap_or_default()
    }

    pub fn set_adapter_config(&mut self, adapter: AdapterType, config: WslAdapterConfig) {
        self.adapters.insert(adapter, config);
    }

    pub fn is_adapter_wsl(&self, adapter: AdapterType) -> bool {
        self.enabled && self.get_adapter_config(adapter).is_wsl()
    }

    pub fn get_wsl_home_dir(&self, adapter: AdapterType) -> Option<PathBuf> {
        if !self.enabled {
            return None;
        }
        self.adapters.get(&adapter).and_then(|c| c.home_dir.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WslDistribution {
    pub name: String,
    pub is_default: bool,
    pub version: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wsl_config_default() {
        let config = WslConfig::default();
        assert!(!config.enabled);
        assert!(config.default_distribution.is_none());
        assert!(config.adapters.is_empty());
    }

    #[test]
    fn test_wsl_adapter_config_windows() {
        let config = WslAdapterConfig::windows();
        assert_eq!(config.mode, WslMode::Windows);
        assert!(!config.is_wsl());
    }

    #[test]
    fn test_wsl_adapter_config_wsl() {
        let config = WslAdapterConfig::wsl(
            "Ubuntu".to_string(),
            PathBuf::from(r"\\wsl$\Ubuntu\home\user"),
        );
        assert_eq!(config.mode, WslMode::Wsl);
        assert!(config.is_wsl());
        assert_eq!(config.distribution, Some("Ubuntu".to_string()));
    }

    #[test]
    fn test_wsl_config_builder() {
        let config = WslConfig::new()
            .with_enabled(true)
            .with_default_distribution("Ubuntu".to_string())
            .with_adapter(
                AdapterType::ClaudeCode,
                WslAdapterConfig::wsl(
                    "Ubuntu".to_string(),
                    PathBuf::from(r"\\wsl$\Ubuntu\home\user"),
                ),
            );

        assert!(config.enabled);
        assert_eq!(config.default_distribution, Some("Ubuntu".to_string()));
        assert!(config.is_adapter_wsl(AdapterType::ClaudeCode));
        assert!(!config.is_adapter_wsl(AdapterType::Cursor));
    }

    #[test]
    fn test_wsl_config_disabled_ignores_wsl_adapters() {
        let config = WslConfig::new().with_enabled(false).with_adapter(
            AdapterType::ClaudeCode,
            WslAdapterConfig::wsl(
                "Ubuntu".to_string(),
                PathBuf::from(r"\\wsl$\Ubuntu\home\user"),
            ),
        );

        assert!(!config.is_adapter_wsl(AdapterType::ClaudeCode));
        assert!(config.get_wsl_home_dir(AdapterType::ClaudeCode).is_none());
    }
}
