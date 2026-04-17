@echo off
chcp 65001

@REM call pnpm tauri build
cd ../client

call npm run tauri build -- --target x86_64-pc-windows-msvc

timeout /t 5 >nul
pause
