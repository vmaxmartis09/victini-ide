@echo off
setlocal
title Cai dat Rust Portable khong can quyen Admin (User Level)

echo ========================================================
echo   VICTINI IDE - Cai dat Rust Portable khong can Admin
echo ========================================================
echo.
echo Script nay tai rustup-init va cai dat truc tiep vao user profile
echo cua Ngai (khong can quyen Administrator cua cong ty).
echo.

where curl >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo [!] Khong tim thay curl. Vui long tai thu cong rustup-init.exe tu:
    echo     https://win.rustup.rs/x86_64
    pause
    exit /b 1
)

echo Dang tai rustup-init.exe...
curl -sSf -L https://win.rustup.rs/x86_64 -o "%TEMP%\rustup-init.exe"

if not exist "%TEMP%\rustup-init.exe" (
    echo [!] Loi tai rustup-init.exe.
    pause
    exit /b 1
)

echo Dang cai dat Rust vao thu muc User...
"%TEMP%\rustup-init.exe" -y --default-host x86_64-pc-windows-msvc --default-toolchain stable

echo.
echo [OK] Da hoan tat cai dat!
echo Vui long mo lai CMD hoac chay run-windows.bat de bat dau code.
del "%TEMP%\rustup-init.exe" >nul 2>nul
pause
