@echo off
setlocal EnableDelayedExpansion
REM Post-build script to rename installer with full timestamp

REM Read version from package.json
for /f "tokens=*" %%a in ('powershell -Command "(Get-Content package.json | ConvertFrom-Json).version"') do set VERSION=%%a

REM Generate full timestamp (YYMMDDHHMM)
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
set FULL_TIMESTAMP=!year!!month!!day!!hour!!minute!

REM Define source and target patterns
set SOURCE_DIR=src-tauri\target\release\bundle

REM Rename NSIS installer if it exists
if exist "%SOURCE_DIR%\nsis\RuleWeaver_*_x64-setup.exe" (
    for %%f in ("%SOURCE_DIR%\nsis\RuleWeaver_*_x64-setup.exe") do (
        set FILENAME=%%~nxf
        set NEWNAME=RuleWeaver_!VERSION!_!FULL_TIMESTAMP!_x64-setup.exe
        echo Renaming !FILENAME! to !NEWNAME!
        rename "%%f" "!NEWNAME!"
    )
)

REM Rename MSI installer if it exists
if exist "%SOURCE_DIR%\msi\RuleWeaver_*_x64_en-US.msi" (
    for %%f in ("%SOURCE_DIR%\msi\RuleWeaver_*_x64_en-US.msi") do (
        set FILENAME=%%~nxf
        set NEWNAME=RuleWeaver_!VERSION!_!FULL_TIMESTAMP!_x64_en-US.msi
        echo Renaming !FILENAME! to !NEWNAME!
        rename "%%f" "!NEWNAME!"
    )
)

echo ✅ Installer renamed with full timestamp: !FULL_TIMESTAMP!
