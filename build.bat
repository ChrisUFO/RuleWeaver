@echo off
setlocal EnableDelayedExpansion
REM RuleWeaver Build Script
REM Builds the production distribution with auto-increment versioning

echo 🔨 Building RuleWeaver...
echo.

REM Get current version from package.json
for /f "tokens=*" %%a in ('powershell -Command "(Get-Content package.json | ConvertFrom-Json).version"') do set CURRENT_VERSION=%%a

REM Extract version components and increment patch
for /f "tokens=1,2,3 delims=- " %%a in ("!CURRENT_VERSION!") do (
    for /f "tokens=1,2,3 delims=." %%i in ("%%a") do (
        set MAJOR=%%i
        set MINOR=%%j
        set PATCH=%%k
    )
)

REM Default to 0.0.0 if parsing fails
if "!MAJOR!"=="" set MAJOR=0
if "!MINOR!"=="" set MINOR=0
if "!PATCH!"=="" set PATCH=0

REM Increment patch with rollover logic (max 255 per component for Windows compatibility)
set /a PATCH=!PATCH!+1
if !PATCH! gtr 255 (
    set PATCH=0
    set /a MINOR=!MINOR!+1
    if !MINOR! gtr 255 (
        set MINOR=0
        set /a MAJOR=!MAJOR!+1
        if !MAJOR! gtr 255 (
            echo ❌ Error: Major version exceeded 255
            exit /b 1
        )
    )
)

REM Generate timestamps
for /f "tokens=2-4 delims=/ " %%a in ('date /t') do (
    set year=%%c
    set month=%%a
    set day=%%b
)
for /f "tokens=1-2 delims=: " %%a in ('time /t') do (
    set hour=%%a
    set minute=%%b
)
set year=!year:~2,2!

REM Full timestamp for filename: YYMMDDHHMM
set FULL_TIMESTAMP=!year!!month!!day!!hour!!minute!

REM Prerelease for version: DDMM (max 3112, fits in 65535)
set PRERELEASE=!day!!month!

REM Final version: MAJOR.MINOR.PATCH-DDMM
set VERSION=!MAJOR!.!MINOR!.!PATCH!-!PRERELEASE!

echo 📦 Current: !CURRENT_VERSION!
echo 📅 New version: !VERSION!
echo.

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

REM Check if node_modules exists
if not exist "node_modules" (
    echo 📦 Installing dependencies...
    call :run_command "npm install"
)

REM Run linting
echo 🔍 Running linters...
call :run_command "npm run lint"
call :run_command "npm run lint:rust"
echo.

REM Run type checks
echo 📋 Running type checks...
call :run_command "npm run typecheck"
echo.

REM Run tests
echo 🧪 Running tests...
call :run_command "npm run test"
call :run_command "npm run test:rust"
echo.

REM Build the application
echo 🏗️  Building production bundle...
call :run_command "npm run tauri:build"

echo.
echo ✅ Build complete! Version: !VERSION!
echo 📁 Distribution files are in src-tauri\target\release\bundle\
goto :eof

:run_command
    call %~1
    if %errorlevel% neq 0 (
        echo Command failed: %~1
        exit /b %errorlevel%
    )
    exit /b 0
