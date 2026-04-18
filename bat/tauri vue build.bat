@echo off
chcp 65001

cd ../client 
call pnpm run build

timeout /t 5 >nul
pause
