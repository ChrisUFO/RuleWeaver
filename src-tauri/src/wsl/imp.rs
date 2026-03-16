//! Windows-specific WSL implementation.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{AppError, Result};
use crate::models::WslDistribution;

/// Provides methods for detecting WSL installations.
pub struct WslDetection;

impl WslDetection {
    /// Check if WSL is installed on the system.
    pub fn is_wsl_installed() -> bool {
        Command::new("wsl.exe")
            .arg("--status")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// List all installed WSL distributions.
    ///
    /// # Errors
    ///
    /// Returns an error if `wsl.exe` cannot be executed or parsing fails.
    pub fn list_distributions() -> Result<Vec<WslDistribution>> {
        let output = Command::new("wsl.exe")
            .args(["--list", "--verbose"])
            .output()
            .map_err(|e| AppError::InvalidInput {
                message: format!("Failed to execute wsl.exe: {}", e),
            })?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = decode_utf16_le(&output.stdout);

        parse_wsl_list_output(&stdout)
    }

    /// Get the default WSL distribution.
    ///
    /// # Errors
    ///
    /// Returns an error if `wsl.exe` cannot be executed.
    pub fn get_default_distribution() -> Result<Option<WslDistribution>> {
        let distributions = Self::list_distributions()?;
        Ok(distributions.into_iter().find(|d| d.is_default))
    }

    /// Get the home directory path for a specific distribution.
    ///
    /// Returns the path in Windows UNC format (e.g., `\\wsl$\Ubuntu\home\user`).
    ///
    /// # Errors
    ///
    /// Returns an error if the home directory cannot be determined.
    pub fn get_home_dir(distribution: &str) -> Result<PathBuf> {
        let output = Command::new("wsl.exe")
            .args(["-d", distribution, "--exec", "echo $HOME"])
            .output()
            .map_err(|e| AppError::InvalidInput {
                message: format!("Failed to execute wsl.exe: {}", e),
            })?;

        if !output.status.success() {
            return Err(AppError::InvalidInput {
                message: format!(
                    "Failed to get home directory for distribution '{}'",
                    distribution
                ),
            });
        }

        let home_path = String::from_utf8_lossy(&output.stdout);
        let home_path = home_path.trim();

        if home_path.is_empty() {
            return Err(AppError::InvalidInput {
                message: format!(
                    "Empty home directory returned for distribution '{}'",
                    distribution
                ),
            });
        }

        Ok(WslPathTranslator::wsl_to_unc(
            Path::new(home_path),
            distribution,
        ))
    }
}

/// Provides methods for translating between Windows UNC and WSL paths.
pub struct WslPathTranslator;

impl WslPathTranslator {
    /// Convert a WSL path to a Windows UNC path.
    ///
    /// # Arguments
    ///
    /// * `wsl_path` - The WSL path (e.g., `/home/user/.claude/CLAUDE.md`)
    /// * `distro` - The WSL distribution name (e.g., "Ubuntu")
    ///
    /// # Returns
    ///
    /// A Windows UNC path (e.g., `\\wsl$\Ubuntu\home\user\.claude\CLAUDE.md`)
    pub fn wsl_to_unc(wsl_path: &Path, distro: &str) -> PathBuf {
        let path_str = wsl_path.to_string_lossy();
        let path_str = path_str.replace('/', "\\");

        if let Some(without_leading_slash) = path_str.strip_prefix('\\') {
            PathBuf::from(format!(r"\\wsl$\{}\{}", distro, without_leading_slash))
        } else if path_str.is_empty() {
            PathBuf::from(format!(r"\\wsl$\{}", distro))
        } else {
            PathBuf::from(format!(r"\\wsl$\{}\{}", distro, path_str))
        }
    }

    /// Convert a Windows UNC path to a WSL path.
    ///
    /// # Arguments
    ///
    /// * `unc_path` - The Windows UNC path (e.g., `\\wsl$\Ubuntu\home\user\file.txt`)
    ///
    /// # Returns
    ///
    /// A tuple of (WSL path, distribution name) if the path is a valid WSL UNC path,
    /// or `None` if it's not a WSL path.
    pub fn unc_to_wsl(unc_path: &Path) -> Option<(PathBuf, String)> {
        let path_str = unc_path.to_string_lossy();
        let path_str = path_str.trim_start_matches(r"\\");

        if !path_str.starts_with("wsl$") && !path_str.starts_with("wsl.localhost") {
            return None;
        }

        let without_prefix = if let Some(stripped) = path_str.strip_prefix("wsl$") {
            stripped
        } else {
            path_str.strip_prefix("wsl.localhost").unwrap_or(path_str)
        };

        let without_prefix = without_prefix.trim_start_matches('\\');

        let (distro, rest) = without_prefix
            .split_once('\\')
            .unwrap_or((without_prefix, ""));

        if distro.is_empty() {
            return None;
        }

        let wsl_path = if rest.is_empty() {
            PathBuf::from("/")
        } else {
            let rest_with_forward_slashes = rest.replace('\\', "/");
            PathBuf::from(format!("/{}", rest_with_forward_slashes))
        };

        Some((wsl_path, distro.to_string()))
    }

    /// Check if a path is a WSL UNC path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to check
    ///
    /// # Returns
    ///
    /// `true` if the path is a WSL UNC path (starts with `\\wsl$\` or `\\wsl.localhost\`)
    pub fn is_wsl_unc(path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        let path_lower = path_str.to_lowercase();
        path_lower.starts_with(r"\\wsl$\") || path_lower.starts_with(r"\\wsl.localhost\")
    }
}

/// Decode UTF-16LE bytes to a String.
///
/// WSL's `--list --verbose` outputs UTF-16LE on Windows. This function handles
/// the conversion robustly, including edge cases like odd-length byte arrays.
///
/// # Arguments
///
/// * `bytes` - The UTF-16LE encoded bytes
///
/// # Returns
///
/// A String with invalid sequences replaced with the Unicode replacement character.
fn decode_utf16_le(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return String::new();
    }

    let u16_vec: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();

    if u16_vec.is_empty() {
        if let Some(&last_byte) = bytes.last() {
            return String::from_utf16_lossy(&[u16::from(last_byte)]);
        }
        return String::new();
    }

    String::from_utf16_lossy(&u16_vec)
}

/// Parse the output of `wsl.exe --list --verbose`.
fn parse_wsl_list_output(output: &str) -> Result<Vec<WslDistribution>> {
    let mut distributions = Vec::new();

    for line in output.lines().skip(1) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let is_default = line.starts_with('*');
        let line = line.trim_start_matches('*').trim();

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let name = parts[0].to_string();
        let version = parts
            .get(2)
            .and_then(|v| v.trim_start_matches('v').parse().ok())
            .unwrap_or(2);

        distributions.push(WslDistribution {
            name,
            is_default,
            version,
        });
    }

    Ok(distributions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wsl_to_unc() {
        let wsl_path = Path::new("/home/user/.claude/CLAUDE.md");
        let unc = WslPathTranslator::wsl_to_unc(wsl_path, "Ubuntu");

        assert_eq!(
            unc.to_string_lossy().to_lowercase(),
            r"\\wsl$\ubuntu\home\user\.claude\claude.md".to_lowercase()
        );
    }

    #[test]
    fn test_wsl_to_unc_root() {
        let wsl_path = Path::new("/home");
        let unc = WslPathTranslator::wsl_to_unc(wsl_path, "Debian");

        assert_eq!(
            unc.to_string_lossy().to_lowercase(),
            r"\\wsl$\debian\home".to_lowercase()
        );
    }

    #[test]
    fn test_unc_to_wsl() {
        let unc_path = Path::new(r"\\wsl$\Ubuntu\home\user\.claude\CLAUDE.md");
        let result = WslPathTranslator::unc_to_wsl(unc_path);

        assert!(result.is_some());
        let (wsl_path, distro) = result.unwrap();
        assert_eq!(wsl_path.to_string_lossy(), "/home/user/.claude/CLAUDE.md");
        assert_eq!(distro, "Ubuntu");
    }

    #[test]
    fn test_unc_to_wsl_localhost() {
        let unc_path = Path::new(r"\\wsl.localhost\Ubuntu\home\user");
        let result = WslPathTranslator::unc_to_wsl(unc_path);

        assert!(result.is_some());
        let (wsl_path, distro) = result.unwrap();
        assert_eq!(wsl_path.to_string_lossy(), "/home/user");
        assert_eq!(distro, "Ubuntu");
    }

    #[test]
    fn test_unc_to_wsl_not_wsl_path() {
        let unc_path = Path::new(r"C:\Users\test");
        let result = WslPathTranslator::unc_to_wsl(unc_path);

        assert!(result.is_none());
    }

    #[test]
    fn test_is_wsl_unc() {
        assert!(WslPathTranslator::is_wsl_unc(Path::new(
            r"\\wsl$\Ubuntu\home"
        )));
        assert!(WslPathTranslator::is_wsl_unc(Path::new(
            r"\\wsl.localhost\Ubuntu\home"
        )));
        assert!(!WslPathTranslator::is_wsl_unc(Path::new(r"C:\Users\test")));
        assert!(!WslPathTranslator::is_wsl_unc(Path::new(r"\\server\share")));
    }

    #[test]
    fn test_parse_wsl_list_output() {
        let output = "  NAME            STATE           VERSION
* Ubuntu          Running         2
  Debian          Stopped         1
";

        let distros = parse_wsl_list_output(output).unwrap();

        assert_eq!(distros.len(), 2);
        assert_eq!(distros[0].name, "Ubuntu");
        assert!(distros[0].is_default);
        assert_eq!(distros[0].version, 2);
        assert_eq!(distros[1].name, "Debian");
        assert!(!distros[1].is_default);
        assert_eq!(distros[1].version, 1);
    }
}
