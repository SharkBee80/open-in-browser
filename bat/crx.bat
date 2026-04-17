echo off
chcp 65001
:: chcp 936 GBK  chcp 65001 UTF-8
title Chrome & Edge 扩展去红色警告
mode con cols=80 lines=25
color 0a
setlocal enabledelayedexpansion

::========== 自动获取管理员权限 ==========
net session >nul 2>&1
if %errorlevel% neq 0 (
  echo.
  echo  正在请求管理员权限...
  powershell -Command "Start-Process '%~f0' -Verb RunAs"
  exit
)
::========== 获取管理员权限 ==========
@REM %1 mshta vbscript:CreateObject("Shell.Application").ShellExecute("cmd.exe","/c %~s0 ::","","runas",1)(window.close)&&exit

goto mode_
::=====菜单=====
:menu_
set choice1=
set choice2=
set iid=
set id=
set mode=
set target=
goto mode_

::=====头=====
:title_1
cls
echo.
echo ╔═══════════════════════════════════════════════════╗ 
echo ║         Chrome / Edge 扩展红色警告去除工具        ║ 
echo ║        Author: SharkBee80  Date: 2025-12-08       ║ 
echo ╚═══════════════════════════════════════════════════╝ 
echo.
exit /b

:=====检测浏览器安装情况=====
:title_2
call :title_1
echo  正在检测浏览器安装情况...
set chrome=0
set edge=0
reg query "HKEY_LOCAL_MACHINE\SOFTWARE\Google\Chrome" >nul 2>&1 && set chrome=1
reg query "HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Edge" >nul 2>&1 && set edge=1

if %chrome%==1 echo  √ 检测到 Google Chrome
if %edge%==1  echo  √ 检测到 Microsoft Edge
echo.
exit /b

::=====选择模式=====
:mode_
call :title_2
echo 请选择模式：
echo.
echo [1] 添加白名单.
echo [2] 删除白名单.
echo [3] 查看白名单.
echo.
echo [9] 返回.
echo [0] 退出.
echo.
set /p choice1=请输入数字后回车:
echo.
if "%choice1%"=="0" goto exity
if "%choice1%"=="1" set mode=add
if "%choice1%"=="2" set mode=remove
if "%choice1%"=="3" set mode=list
if "%choice1%"=="9" goto menu_
if "%choice1%"=="" echo 输入错误,请重新选！ & timeout /t 1 >nul & echo. & goto mode_

if "%mode%"=="add" echo √ 已选择 添加白名单.
if "%mode%"=="remove" echo √ 已选择 删除白名单.
if "%mode%"=="list" echo √ 已选择 查看白名单.
if "%mode%"=="" echo 输入错误,请重新选！ & timeout /t 1 >nul & echo. & goto mode_
timeout /t 1 >nul
goto target_

::=====选择浏览器=====
:target_
call :title_2
echo 请选择要操作的浏览器：
echo.
echo [1] Google Chrome.
echo [2] Microsoft Edge.
echo [3] Chrome 和 Edge (推荐).
echo.
echo [9] 返回.
echo [0] 退出.
echo.
set /p choice2=请输入数字后回车:
echo.
if "%choice2%"=="0" exit
if "%choice2%"=="1" set target=chrome& 
if "%choice2%"=="2" set target=edge&
if "%choice2%"=="3" set target=both&
if "%choice2%"=="9" cls & goto mode_
if "%choice2%"=="" echo 输入错误,请重新选！ & timeout /t 1 >nul & echo. & goto target_

if "%target%"=="chrome" echo √ 已选择 Google Chrome.
if "%target%"=="edge" echo √ 已选择 Microsoft Edge.
if "%target%"=="both" echo √ 已选择 Chrome 和 Edge.
if "%target%"=="" echo 输入错误,请重新选！ & timeout /t 1 >nul & echo. & goto target_
timeout /t 1 >nul
goto route_0
::=====导航路径=====
:route_0
if "%mode%"=="add" goto input_
if "%mode%"=="remove" goto input_
if "%mode%"=="list" goto list_
::=====导航路径=====
:route_1
if "%mode%"=="add" goto add_
if "%mode%"=="remove" goto remove_
if "%mode%"=="list" goto list_
::======输入ID======
:input_
cls
echo.
call :title_1
echo 请打开浏览器 → 地址栏输入以下地址 → 开启右上角「开发者模式」.
echo Chrome 用: chrome://extensions/.
echo Edge 用: edge://extensions/.
echo.
echo 找到你的扩展,把那串 32 位 ID 复制下来(一串小写字母).
echo 示例: eljjpcjekbenlpmmlkoeigmcimnaaimn
echo.
set /p iid=粘贴你的扩展 ID 到这里后按回车:
set id=%iid%
if "%id%"=="quit" goto menu_
if "%id%"=="" echo ID 不能为空！ & timeout /t 1 >nul & goto input_
call :len_check %id% 32
if %arg%==1 goto route_1
@REM goto route_1
echo 错误: ID 必须正好 32 位!你输入的长度不对,请重新复制.
timeout /t 3 >nul
goto input_

::=====添加=====
:add_
echo.
call :title_1
echo 正在写入注册表，请稍等...

::========== 处理 Chrome ==========
if %chrome%==1 if "%target%"=="chrome" (
  call :add_to_registry "HKEY_LOCAL_MACHINE\SOFTWARE\Policies\Google\Chrome" "%id%"
  call :white_chrome_in
)
if %chrome%==1 if "%target%"=="both" (
  call :add_to_registry "HKEY_LOCAL_MACHINE\SOFTWARE\Policies\Google\Chrome" "%id%"
  call :white_chrome_in
)

::========== 处理 Edge ==========
if %edge%==1 if "%target%"=="edge" (
  call :add_to_registry "HKEY_LOCAL_MACHINE\SOFTWARE\Policies\Microsoft\Edge" "%id%"
  call :white_edge_in
)
if %edge%==1 if "%target%"=="both" (
  call :add_to_registry "HKEY_LOCAL_MACHINE\SOFTWARE\Policies\Microsoft\Edge" "%id%"
  call :white_edge_in
)

echo ███████  全部完成！ 
echo.
echo 请立刻重启 Chrome / Edge, 红色警告将消失!
echo.
echo 按任意键返回菜单
pause
timeout /t 1.5 >nul
goto menu_

::=====写入chrome=====
:white_chrome_in
if not %arg%==1 (
  echo ████  ID ⌈%id%⌋ 已成功加入 chrome 白名单！ 
) else (
  echo ␣␣␣␣␣ ID ⌈%id%⌋ 已存在于 Chrome 白名单中
)
echo.
exit /b

::======写入edge======
:white_edge_in
if not %arg%==1 (
  echo ████  ID ⌈%id%⌋ 已成功加入 Edge 白名单！
) else (
  echo ␣␣␣␣␣ ID ⌈%id%⌋ 已存在于 Edge 白名单中
)

echo.
exit /b

::========== 子程序：自动找下一个空位写入 ==========
:add_to_registry
set key=%~1
set newid=%~2
set arg=0
set found=0

set count=0


for /f "skip=2 tokens=2,*" %%A in ('reg query "%key%\ExtensionInstallAllowlist" 2^>nul') do (
  if /i "%%B"=="%id%" set found=1
)

if %found%==1 set arg=1 & exit /b

:loop
set /a count+=1
reg query "%key%\ExtensionInstallAllowlist" /v %count% >nul 2>&1
if %errorlevel%==0 goto loop

reg add "%key%\ExtensionInstallAllowlist" /v %count% /t REG_SZ /d %newid% /f >nul

if "%key:Google=%"=="%key%" (
  echo  → [%count%] ⌈%newid%⌋ 已加入 Edge 白名单.
  ) else (
  echo  → [%count%] ⌈%newid%⌋ 已加入 Chrome 白名单.
)
exit /b

::=====移除=====
:remove_
echo.
call :title_1
echo 正在删除注册表，请稍等...
::========== 处理 Chrome ==========
if %chrome%==1 if "%target%"=="chrome" (
  call :remove_from_registry "HKEY_LOCAL_MACHINE\SOFTWARE\Policies\Google\Chrome" %id%
  call :white_chrome_out
)
if %chrome%==1 if "%target%"=="both" (
  call :remove_from_registry "HKEY_LOCAL_MACHINE\SOFTWARE\Policies\Google\Chrome" %id%
  call :white_chrome_out
)
::========== 处理 Edge ==========
if %edge%==1 if "%target%"=="edge" (
  call :remove_from_registry "HKEY_LOCAL_MACHINE\SOFTWARE\Policies\Microsoft\Edge" %id%
  call :white_edge_out
)
if %edge%==1 if "%target%"=="both" (
  call :remove_from_registry "HKEY_LOCAL_MACHINE\SOFTWARE\Policies\Microsoft\Edge" %id%
  call :white_edge_out
)

echo ███████  全部完成!
echo.
echo 按任意键返回菜单
pause
timeout /t 1.5 >nul
goto menu_

::=====移出chrome=====
:white_chrome_out
if not %arg%==1 (
  echo ████  ID ⌈%id%⌋ 已成功移出 chrome 白名单！
) else (
  echo ␣␣␣␣␣ ID ⌈%id%⌋ 不存在于 Chrome 白名单中
)
echo.
exit /b

::======移出edge======
:white_edge_out
if not %arg%==1 (
  echo ████  ID ⌈%id%⌋ 已成功移出 Edge 白名单！
) else (
  echo ␣␣␣␣␣ ID ⌈%id%⌋ 不存在于 Edge 白名单中
)
echo.
exit /b

::========== 子程序：自动寻找并移除 ==========
:remove_from_registry
set key=%~1
set newid=%~2
set arg=0
set found=0

for /f "skip=2 tokens=2,*" %%A in ('reg query "%key%\ExtensionInstallAllowlist" 2^>nul') do (
  if /i "%%B"=="%newid%" set found=1
)

if %found%==0 set arg=1 & exit /b

for /f "tokens=1,2*" %%a in ('reg query "%key%\ExtensionInstallAllowlist" 2^>nul') do (
    rem %%a=键名, %%b=类型(REG_SZ等), %%c=值数据
    if not "%%c"=="" (
        rem 去掉值数据两边的空格
        set "value=%%c"
        if "!value!"=="!newid!" (
            @REM echo [找到] 键名: %%a  值: !value!
            @REM echo.
            reg delete "%key%\ExtensionInstallAllowlist" /v %%a /f >nul
        )
    )
)

if "%key:Google=%"=="%key%" (
  echo  → ⌈%newid%⌋ 已移出 Edge 白名单.
  ) else (
  echo  → ⌈%newid%⌋ 已移出 Chrome 白名单.
)
exit /b

::======== 查看白名单 ========
:list_
cls
call :title_1
if "%target%"=="chrome" echo 已选择 Google Chrome 白名单列表：
if "%target%"=="edge" echo 已选择 Microsoft Edge 白名单列表：
if "%target%"=="both" echo 已选择 Chrome 和 Edge 白名单列表：
call :list_all %target%

echo  按任意键返回菜单
pause
timeout /t 1.5 >nul
goto menu_

::=====列出=====
:list_all
echo.
if "%~1"=="chrome" call :list_one "HKEY_LOCAL_MACHINE\SOFTWARE\Policies\Google\Chrome" "Chrome"
if "%~1"=="edge"   call :list_one "HKEY_LOCAL_MACHINE\SOFTWARE\Policies\Microsoft\Edge" "Edge"
if "%~1"=="both" (
    call :list_one "HKEY_LOCAL_MACHINE\SOFTWARE\Policies\Google\Chrome" "Chrome"
    call :list_one "HKEY_LOCAL_MACHINE\SOFTWARE\Policies\Microsoft\Edge" "Edge"
)
exit /b
::=====子程序：列出指定注册表下的所有值 =====
:list_one
set key=%~1
set name=%~2

echo [%name%]
reg query "%key%\ExtensionInstallAllowlist" >nul 2>&1 || (
    echo ⌈ 空 ⌋
    echo.
    exit /b
)

for /f "skip=2 tokens=2,*" %%A in ('reg query "%key%\ExtensionInstallAllowlist" 2^>nul') do echo  %%B
echo.
exit /b

::=====长度检测=====
:len_check
set str=%~1
set len=%~2
set /a len2=len-1
set arg=0
set a=0
@REM echo !str:~%len2%,1!
if not "!str:~%len2%,1!"=="" set /a a+=1
@REM echo !str:~%len%,1!
if "!str:~%len%,1!"=="" set /a a+=1
@REM echo !a!
if "!a!"=="2" set arg=1
@REM echo %arg%
exit /b

:exity
exit /b
exit
