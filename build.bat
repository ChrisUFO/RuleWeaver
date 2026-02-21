@echo off
REM RuleWeaver Build Script
REM Builds the production distribution

echo 🔨 Building RuleWeaver...
echo.

REM Check if node_modules exists
if not exist "node_modules" (
    echo 📦 Installing dependencies...
    npm install
    echo.
)

REM Run linting
echo 🔍 Running linters...
call npm run lint
if %errorlevel% neq 0 exit /b %errorlevel%
call npm run lint:rust
if %errorlevel% neq 0 exit /b %errorlevel%
echo.

REM Run type checks
echo 📋 Running type checks...
call npm run typecheck
if %errorlevel% neq 0 exit /b %errorlevel%
echo.

REM Run tests
echo 🧪 Running tests...
call npm run test
if %errorlevel% neq 0 exit /b %errorlevel%
call npm run test:rust
if %errorlevel% neq 0 exit /b %errorlevel%
echo.

REM Build the application
echo 🏗️  Building production bundle...
call npm run tauri:build
if %errorlevel% neq 0 exit /b %errorlevel%

echo.
echo ✅ Build complete!
echo 📁 Distribution files are in src-tauri\target\release\bundle\
