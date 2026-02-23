@echo off
setlocal EnableDelayedExpansion
REM Set timestamp-based version across all config files
REM Usage: scripts\set-version.bat [version]
REM If no version provided, uses current timestamp (YYMMDD.HHMM)

if "%~1"=="" (
    REM Generate timestamp version (YYMMDD.HHMM format)
    for /f "tokens=2-4 delims=/ " %%a in ('date /t') do (
        set year=%%c
        set month=%%a
        set day=%%b
    )
    for /f "tokens=1-2 delims=: " %%a in ('time /t') do (
        set hour=%%a
        set minute=%%b
    )
    REM Remove century from year (20) and add 0. prefix for semver
    set year=!year:~2,2!
    set VERSION=0.!year!!month!!day!.!hour!!minute!
) else (
    set VERSION=%~1
)

echo 📅 Setting version to: !VERSION!

REM Update package.json version
powershell -Command "(Get-Content package.json) -replace '\"version\": \"[^\"]+\"', '\"version\": \"!VERSION!\"' | Set-Content package.json"
echo ✓ Updated package.json

REM Update Cargo.toml version
powershell -Command "(Get-Content src-tauri\Cargo.toml) -replace '^version = \"[^\"]+\"', 'version = \"!VERSION!\"' | Set-Content src-tauri\Cargo.toml"
echo ✓ Updated Cargo.toml

REM Update tauri.conf.json version
powershell -Command "(Get-Content src-tauri\tauri.conf.json) -replace '\"version\": \"[^\"]+\"', '\"version\": \"!VERSION!\"' | Set-Content src-tauri\tauri.conf.json"
echo ✓ Updated tauri.conf.json

REM Create/update version.json for frontend access
echo { "version": "!VERSION!", "timestamp": "!date! !time!" } > public\version.json
echo ✓ Updated public\version.json

echo.
echo ✅ Version set to !VERSION!
