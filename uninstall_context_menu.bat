@echo off
echo Uninstalling File Lock Checker Context Menu...
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

echo Removing context menu entries...

REM ลบ registry entries สำหรับ All Files (*)
reg delete "HKCR\*\shell\FLC_Check" /f >nul 2>&1
reg delete "HKCR\*\shell\FLC_Unlock" /f >nul 2>&1
reg delete "HKCR\*\shell\FLC_Monitor" /f >nul 2>&1

REM ลบ registry entries สำหรับ Directories
reg delete "HKCR\Directory\shell\FLC_Check" /f >nul 2>&1
reg delete "HKCR\Directory\shell\FLC_Monitor" /f >nul 2>&1

echo.
echo SUCCESS: Context menu uninstalled successfully!
echo The FLC menu items have been removed from right-click context menu.
echo.
pause