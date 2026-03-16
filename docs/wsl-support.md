# WSL Support

RuleWeaver supports syncing rules, commands, and skills to WSL (Windows Subsystem for Linux) distributions. This allows AI tools running inside WSL to access RuleWeaver-managed configurations.

## Requirements

- Windows 10 version 2004+ or Windows 11
- WSL 2 installed and configured
- At least one WSL distribution installed

## Setup

### 1. Enable WSL Support

1. Open RuleWeaver Settings (Ctrl+,)
2. Navigate to the **Infrastructure** tab
3. Find the **WSL Support** card
4. Toggle **Enable WSL Support** to on

### 2. Select Default Distribution

When WSL support is enabled, you can select your default WSL distribution from the dropdown. RuleWeaver will auto-detect installed distributions.

### 3. Configure Per-Adapter Settings

By default, all adapters will sync to Windows paths. To sync specific adapters to WSL:

1. In Settings, go to the **Capabilities** tab
2. Find the adapter you want to configure for WSL
3. The adapter will use the default WSL distribution when enabled

## How It Works

### UNC Paths

RuleWeaver uses Windows UNC paths to access WSL filesystems:

```
\\wsl$\Ubuntu\home\user\.claude\CLAUDE.md
\\wsl.localhost\Ubuntu\home\user\.claude\CLAUDE.md
```

Both `\\wsl$\` and `\\wsl.localhost\` formats are supported.

### Path Translation

| Windows Path                       | WSL Path              |
| ---------------------------------- | --------------------- |
| `\\wsl$\Ubuntu\home\user\file.txt` | `/home/user/file.txt` |
| `\\wsl.localhost\Debian\home\user` | `/home/user`          |

### Sync Behavior

When WSL support is enabled:

1. RuleWeaver detects all installed WSL distributions
2. Rules are synced to the configured WSL distribution's home directory
3. File operations use Windows UNC paths (no WSL interop required)
4. Changes are immediately visible in WSL

## Per-Adapter Configuration

You can configure different adapters to use different targets:

- **Claude Code** → Windows (`C:\Users\...`)
- **Cline** → WSL Ubuntu (`\\wsl$\Ubuntu\home\...`)
- **Cursor** → WSL Debian (`\\wsl$\Debian\home\...`)

This allows mixed environments where some AI tools run on Windows and others run in WSL.

## Performance Considerations

### UNC Path Performance

UNC paths to WSL have some performance characteristics to be aware of:

- **Read Performance**: Generally fast for small files (rules, commands)
- **Write Performance**: Slightly slower than native Windows paths
- **Large Files**: Not recommended for large binary files

### Recommendations

1. **Use WSL 2**: WSL 2 has significantly better filesystem performance than WSL 1
2. **Keep Rules Small**: Individual rule files are small, so sync performance is good
3. **Disable When Not Needed**: If you don't use WSL, keep the feature disabled

## Troubleshooting

### WSL Not Detected

If RuleWeaver doesn't detect your WSL installation:

1. Verify WSL is installed: `wsl --list --verbose`
2. Ensure at least one distribution is installed
3. Try restarting RuleWeaver

### Permission Denied

If you get permission errors when syncing to WSL:

1. Check the WSL distribution is running: `wsl -l -v`
2. Verify the path exists in WSL
3. Check file permissions inside WSL

### Paths Not Syncing

1. Verify WSL support is enabled in Settings
2. Check the default distribution is correct
3. Try a manual sync from the Rules page

### Slow Sync Performance

1. Ensure you're using WSL 2 (not WSL 1)
2. Check if Windows Defender is scanning UNC paths
3. Consider excluding the WSL path from antivirus scans

## Technical Details

### WSL Detection

RuleWeaver uses `wsl.exe --list --verbose` to detect installed distributions. The output is parsed to extract:

- Distribution name
- Default distribution indicator
- WSL version (1 or 2)
- State (Running/Stopped)

### Configuration Storage

WSL configuration is stored in the RuleWeaver database under the `wsl_config` key:

```json
{
  "enabled": true,
  "defaultDistribution": "Ubuntu",
  "adapters": {
    "claudeCode": {
      "mode": "wsl",
      "distribution": "Ubuntu",
      "homeDir": "\\\\wsl$\\Ubuntu\\home\\user"
    }
  }
}
```

### Code Architecture

All WSL-specific code is gated with `#[cfg(target_os = "windows")]`:

- `src-tauri/src/wsl/mod.rs` - Module entry point
- `src-tauri/src/wsl/imp.rs` - Windows-specific implementation
- `src-tauri/src/models/wsl.rs` - Configuration models

This ensures no impact on macOS/Linux builds.
