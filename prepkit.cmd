@echo off
rem PrepKit CLI launcher for scripts and AI agents: prepkit.cmd <command> [options]   (prepkit.cmd --help)
rem Uses a built prepkit-cli.exe, builds one with cargo if Rust is installed, or downloads the latest release.
setlocal
set "ROOT=%~dp0"

set "EXE=%ROOT%target\release\prepkit-cli.exe"
if exist "%EXE%" goto run
set "EXE=%ROOT%bin\prepkit-cli.exe"
if exist "%EXE%" goto run

where cargo >nul 2>&1
if errorlevel 1 goto download
echo Building prepkit-cli (first run only)... 1>&2
cargo build --release --quiet -p prepkit-cli --manifest-path "%ROOT%Cargo.toml" 1>&2
if errorlevel 1 exit /b 1
set "EXE=%ROOT%target\release\prepkit-cli.exe"
goto run

:download
echo Downloading prepkit-cli from the latest GitHub release... 1>&2
powershell -NoProfile -ExecutionPolicy Bypass -Command "$ErrorActionPreference='Stop'; New-Item -ItemType Directory -Force '%ROOT%bin' | Out-Null; Invoke-WebRequest 'https://github.com/nyaridev/prepkit/releases/latest/download/prepkit-cli.exe' -OutFile '%ROOT%bin\prepkit-cli.exe'" 1>&2
if errorlevel 1 (
    echo Couldn't download prepkit-cli. Install Rust from https://rustup.rs and run this again, or get prepkit-cli.exe from https://github.com/nyaridev/prepkit/releases 1>&2
    exit /b 1
)
set "EXE=%ROOT%bin\prepkit-cli.exe"

:run
"%EXE%" %*
exit /b %errorlevel%
