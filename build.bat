@echo off
setlocal
REM RuleWeaver Build Script
REM Builds the production distribution

echo 🔨 Building RuleWeaver...
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
echo ✅ Build complete!
echo 📁 Distribution files are in src-tauri\target\release\bundle\
goto :eof

:run_command
    call %~1
    if %errorlevel% neq 0 (
        echo Command failed: %~1
        exit /b %errorlevel%
    )
    exit /b 0
