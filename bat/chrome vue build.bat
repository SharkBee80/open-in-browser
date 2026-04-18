@echo off
chcp 65001

cd ../extension
call pnpm run build

timeout /t 5 >nul
pause