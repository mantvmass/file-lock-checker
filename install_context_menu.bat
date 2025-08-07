@echo off
echo Installing File Lock Checker Context Menu...
echo.

REM ตรวจสอบว่ารันเป็น Administrator หรือไม่
net session >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: This script must be run as Administrator!
    echo Right-click and select "Run as administrator"
    echo.
    pause
    exit /b 1
)

REM หา path ของ file-lock-checker.exe
set "EXE_PATH=%~dp0file-lock-checker.exe"

if not exist "%EXE_PATH%" (
    echo ERROR: file-lock-checker.exe not found in current directory!
    echo Please make sure file-lock-checker.exe is in the same folder as this script.
    echo.
    pause
    exit /b 1
)

echo Found executable: %EXE_PATH%
echo.

REM สร้าง registry entries สำหรับ All Files (*)
echo Installing context menu for all files...

REM FLC Check
reg add "HKCR\*\shell\FLC_Check" /ve /d "FLC Check" /f >nul
reg add "HKCR\*\shell\FLC_Check" /v "Icon" /d "%EXE_PATH%,0" /f >nul
reg add "HKCR\*\shell\FLC_Check\command" /ve /d "\"%EXE_PATH%\" check \"%%1\"" /f >nul

REM FLC Unlock
reg add "HKCR\*\shell\FLC_Unlock" /ve /d "FLC Unlock" /f >nul
reg add "HKCR\*\shell\FLC_Unlock" /v "Icon" /d "%EXE_PATH%,0" /f >nul
reg add "HKCR\*\shell\FLC_Unlock\command" /ve /d "\"%EXE_PATH%\" unlock \"%%1\"" /f >nul

REM FLC Monitor
reg add "HKCR\*\shell\FLC_Monitor" /ve /d "FLC Monitor" /f >nul
reg add "HKCR\*\shell\FLC_Monitor" /v "Icon" /d "%EXE_PATH%,0" /f >nul
reg add "HKCR\*\shell\FLC_Monitor\command" /ve /d "\"%EXE_PATH%\" monitor \"%%1\"" /f >nul

REM สร้าง registry entries สำหรับ Directories
echo Installing context menu for directories...

REM FLC Check for folders
reg add "HKCR\Directory\shell\FLC_Check" /ve /d "FLC Check" /f >nul
reg add "HKCR\Directory\shell\FLC_Check" /v "Icon" /d "%EXE_PATH%,0" /f >nul
reg add "HKCR\Directory\shell\FLC_Check\command" /ve /d "\"%EXE_PATH%\" check \"%%1\"" /f >nul

REM FLC Monitor for folders
reg add "HKCR\Directory\shell\FLC_Monitor" /ve /d "FLC Monitor" /f >nul
reg add "HKCR\Directory\shell\FLC_Monitor" /v "Icon" /d "%EXE_PATH%,0" /f >nul
reg add "HKCR\Directory\shell\FLC_Monitor\command" /ve /d "\"%EXE_PATH%\" monitor \"%%1\"" /f >nul

echo.
echo SUCCESS: Context menu installed successfully!
echo.
echo You can now right-click on any file or folder to see:
echo - FLC Check   (Check which processes are locking the file)
echo - FLC Unlock  (Kill processes that are locking the file)
echo - FLC Monitor (Monitor file locks in real-time)
echo.
echo To uninstall, run: uninstall_context_menu.bat
echo.
pause