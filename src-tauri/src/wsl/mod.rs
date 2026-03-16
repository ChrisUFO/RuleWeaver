//! WSL (Windows Subsystem for Linux) support module.
//!
//! This module provides utilities for detecting WSL installations and
//! translating paths between Windows and WSL formats.
//!
//! All functionality is gated to Windows builds only.

#[cfg(target_os = "windows")]
mod imp;

#[cfg(target_os = "windows")]
pub use imp::*;

/// Stub implementations for non-Windows platforms.
/// These always return empty/not-found results since WSL doesn't exist.
#[cfg(not(target_os = "windows"))]
pub struct WslDistribution {
    pub name: String,
    pub is_default: bool,
    pub version: u8,
}

#[cfg(not(target_os = "windows"))]
pub struct WslDetection;

#[cfg(not(target_os = "windows"))]
pub struct WslPathTranslator;

#[cfg(not(target_os = "windows"))]
pub fn validate_distribution_name(name: &str) -> crate::error::Result<()> {
    if name.is_empty() {
        return Err(crate::error::AppError::InvalidInput {
            message: "Distribution name cannot be empty".to_string(),
        });
    }
    let valid = name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_');
    if !valid {
        return Err(crate::error::AppError::InvalidInput {
            message: format!(
                "Invalid distribution name '{}': must contain only alphanumeric characters, hyphens, or underscores",
                name
            ),
        });
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
impl WslDetection {
    pub fn is_wsl_installed() -> bool {
        false
    }

    pub fn list_distributions() -> crate::error::Result<Vec<WslDistribution>> {
        Ok(Vec::new())
    }

    pub fn get_default_distribution() -> crate::error::Result<Option<WslDistribution>> {
        Ok(None)
    }

    pub fn get_home_dir(_distribution: &str) -> crate::error::Result<std::path::PathBuf> {
        Err(crate::error::AppError::InvalidInput {
            message: "WSL is not available on this platform".to_string(),
        })
    }
}

#[cfg(not(target_os = "windows"))]
impl WslPathTranslator {
    pub fn wsl_to_unc(wsl_path: &std::path::Path, _distro: &str) -> std::path::PathBuf {
        wsl_path.to_path_buf()
    }

    pub fn unc_to_wsl(_unc_path: &std::path::Path) -> Option<(std::path::PathBuf, String)> {
        None
    }

    pub fn is_wsl_unc(_path: &std::path::Path) -> bool {
        false
    }
}
