# WSL Path Performance Research

## Overview

This document captures research findings on accessing WSL files from Windows via UNC paths for the WSL Support feature (Milestone #21).

## Access Methods

### 1. UNC Path Access (Recommended Approach)

Windows can access WSL filesystems via UNC paths:

- Format: `\\wsl$\{distribution}\{path}`
- Example: `\\wsl$\Ubuntu\home\user\.claude\CLAUDE.md`

**Advantages:**

- Native Windows file API support
- No need to shell out to `wsl.exe` for file operations
- Works with standard Rust `std::fs` operations
- Tauri's fs API handles UNC paths correctly

**Performance Characteristics:**

- Slightly slower than native Windows paths due to protocol translation
- WSL 2 uses 9P protocol for file access (more overhead than WSL 1)
- Acceptable for config file sizes (typically < 100KB)
- Initial connection has minor latency, subsequent operations are faster

### 2. wsl.exe Command Execution

Alternative approach using `wsl.exe` for file operations:

- Example: `wsl.exe -d Ubuntu cat /home/user/.claude/CLAUDE.md`

**Disadvantages:**

- Process spawn overhead per operation
- More complex error handling
- Not suitable for frequent file operations
- Better reserved for detection/queries only

## Technical Findings

### UNC Path Handling in Rust

```rust
use std::path::PathBuf;

// UNC paths work with standard PathBuf
let unc_path = PathBuf::from(r"\\wsl$\Ubuntu\home\user\.claude\CLAUDE.md");

// Standard fs operations work
std::fs::read_to_string(&unc_path)?;
std::fs::write(&unc_path, content)?;
```

### Path Translation

| Path Type       | Format                   | Example                                     |
| --------------- | ------------------------ | ------------------------------------------- |
| WSL Native      | `/home/user/file`        | `/home/user/.claude/CLAUDE.md`              |
| Windows UNC     | `\\wsl$\{distro}\{path}` | `\\wsl$\Ubuntu\home\user\.claude\CLAUDE.md` |
| Windows Mounted | `/mnt/c/Users/...`       | `/mnt/c/Users/chris/file.txt`               |

### WSL Distribution Detection

Use `wsl.exe` commands for detection:

- `wsl.exe --list --verbose` - List distributions with status
- `wsl.exe --exec echo $HOME` - Get home directory for a distribution

## Recommendations

1. **Use UNC paths for file sync** - Direct file operations via UNC paths are performant enough for config file sizes
2. **Cache distribution list** - Detect distributions once, cache for session
3. **Use wsl.exe only for detection** - Reserve process spawning for initial setup
4. **Handle connection errors gracefully** - WSL may not be running, prompt user to start it
5. **Test on WSL 1 and WSL 2** - Both have different performance characteristics

## Implementation Decision

Based on this research, we will:

1. Use UNC paths (`\\wsl$\...`) for all file sync operations
2. Use `wsl.exe --list --verbose` for distribution detection
3. Use `wsl.exe --exec echo $HOME` for home directory detection
4. Implement graceful error handling for WSL not running scenarios

## Limitations

- UNC paths require WSL to be running (at least one distribution)
- File watching over UNC paths may have limitations (not critical for sync-on-save)
- Some edge cases with special characters in distribution names
