@echo off
chcp 65001

cd ../client 
call pnpm run tauri dev

timeout /t 5 >nul
pause
