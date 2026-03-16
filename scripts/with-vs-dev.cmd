@echo off
setlocal

set "VS_DEV_CMD=%ProgramFiles(x86)%\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat"
if not exist "%VS_DEV_CMD%" (
  echo Visual Studio Build Tools environment not found at "%VS_DEV_CMD%".
  exit /b 1
)

call "%VS_DEV_CMD%" -arch=x64 >nul
if errorlevel 1 (
  exit /b %errorlevel%
)

set "PATH=%USERPROFILE%\.cargo\bin;%LOCALAPPDATA%\Microsoft\WinGet\Links;%PATH%"
call %*
exit /b %errorlevel%
