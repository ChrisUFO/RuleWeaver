@echo off
setlocal EnableDelayedExpansion
REM RuleWeaver Build Script
REM Builds the production distribution with timestamp-based versioning

echo 🔨 Building RuleWeaver...
echo.

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
REM Remove century from year (20) and leading zeros if any
set year=!year:~2,2!
set month=!month:~0,2!
set day=!day:~0,2!
set hour=!hour:~0,2!
set minute=!minute:~0,2!

set VERSION=!year!!month!!day!.!hour!!minute!
echo 📅 Setting version to: !VERSION!
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
