@echo off
setlocal enabledelayedexpansion
title Victini IDE - Portable Environment Checker

echo ========================================================
echo   VICTINI IDE - Autonomous Vibe-Code Station
echo ========================================================
echo.

:: 1. Check if prebuilt executable exists
if exist "bin\victini.exe" (
    echo [OK] Tim thay ban chay san: bin\victini.exe
    echo Dang khoi chay Victini IDE...
    start "" "bin\victini.exe"
    exit /b 0
)

if exist "target\release\victini.exe" (
    echo [OK] Tim thay ban release: target\release\victini.exe
    echo Dang khoi chay Victini IDE...
    start "" "target\release\victini.exe"
    exit /b 0
)

:: 2. Check if local portable toolchain exists
if exist "tools\cargo\bin\cargo.exe" (
    echo [OK] Su dung Portable Rust Toolchain trong thu muc tools...
    set "PATH=%CD%\tools\cargo\bin;%CD%\tools\rustc\bin;%PATH%"
    cargo run --release
    exit /b 0
)

:: 3. Check system cargo
where cargo >nul 2>nul
if %ERRORLEVEL% equ 0 (
    echo [OK] May da co san Cargo/Rust he thong.
    cargo run --release
    exit /b 0
)

echo [!] Khong tim thay ban chay san hoac Rust.
echo     Vui long kiem tra file bin\victini.exe hoac tools\setup-portable-rust.bat
pause
