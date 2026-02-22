@echo off
REM RuleWeaver Quick Build Script
REM Builds without running tests/linting (for faster iteration)

echo ⚡ Quick building RuleWeaver...
call npm run tauri:build

echo.
echo ✅ Build complete!
echo 📁 Distribution files are in src-tauri\target\release\bundle\
