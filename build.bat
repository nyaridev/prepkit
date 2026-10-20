@echo off
rem Builds the release files into release\: PrepKit.exe (the app) and prepkit-cli.exe (the CLI).
setlocal
cd /d "%~dp0"

call pnpm install --frozen-lockfile || goto :fail
call pnpm tauri build --no-bundle || goto :fail
cargo build --release -p prepkit-cli || goto :fail

if not exist release mkdir release
copy /y target\release\prepkit.exe release\PrepKit.exe >nul || goto :fail
copy /y target\release\prepkit-cli.exe release\prepkit-cli.exe >nul || goto :fail
echo.
echo Built release\PrepKit.exe and release\prepkit-cli.exe
exit /b 0

:fail
echo Build failed.
exit /b 1
