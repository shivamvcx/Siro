@echo off
setlocal EnableDelayedExpansion
title Siro Desktop Manager

:: Ensure current working directory is the repository root
cd /d "%~dp0"

:: If an argument was provided (e.g. "siro.bat 1"), process directly
if not "%~1"=="" (
    set "CHOICE=%~1"
    goto PROCESS_CHOICE
)

:MENU
cls
echo ======================================================================
echo                     SIRO DESKTOP CONTROLLER
echo ======================================================================
echo.
echo   [1] Build and Launch (Debug)       - Auto-closes running app, rebuilds and runs
echo   [2] Build and Launch (Release)     - Auto-closes running app, runs 60fps release
echo   [3] Fast Launch (No Rebuild)       - Instantly opens pre-compiled binary (^<1s)
echo   [4] Full Verification              - Runs typecheck (cargo check) + all tests
echo   [5] Kill Running App Only          - Force-closes any background siro.exe
echo   [6] Clean Build Cache              - Safe clean of target/ (source files untouched)
echo   [7] Package Release Deliverables   - Builds portable binary + installers in dist/
echo   [Q] Exit
echo.
echo ======================================================================
set "CHOICE="
set /p "CHOICE=Enter choice [1-7, Q]: "

:PROCESS_CHOICE
:: Strip any leading or trailing spaces from input
if defined CHOICE set "CHOICE=!CHOICE: =!"

if "%CHOICE%"=="1" goto RUN_DEBUG
if "%CHOICE%"=="2" goto RUN_RELEASE
if "%CHOICE%"=="3" goto FAST_LAUNCH
if "%CHOICE%"=="4" goto VERIFY_ALL
if "%CHOICE%"=="5" goto KILL_APP
if "%CHOICE%"=="6" goto CLEAN_CACHE
if "%CHOICE%"=="7" goto PACKAGE_RELEASE
if /i "%CHOICE%"=="Q" goto EXIT_SCRIPT

echo.
echo [!] Invalid choice "%CHOICE%". Please try again.
ping -n 2 127.0.0.1 >nul
goto MENU

:RUN_DEBUG
echo.
echo ======================================================================
echo [1/2] Checking for running instances of Siro...
echo ======================================================================
tasklist /fi "imagename eq siro.exe" 2>nul | find /i "siro.exe" >nul
if not errorlevel 1 (
    echo [*] Detected running siro.exe instance. Closing it now...
    taskkill /F /IM siro.exe /T >nul 2>&1
    ping -n 2 127.0.0.1 >nul
    echo [*] Closed successfully.
) else (
    echo [*] No running instance found.
)

echo.
echo ======================================================================
echo [2/2] Building and launching Siro (Debug mode)...
echo ======================================================================
cargo run --bin siro
if errorlevel 1 (
    echo.
    echo [!] Build or execution failed with error code %errorlevel%.
    echo Press any key to return to menu...
    pause >nul
)
goto MENU

:RUN_RELEASE
echo.
echo ======================================================================
echo [1/2] Checking for running instances of Siro...
echo ======================================================================
tasklist /fi "imagename eq siro.exe" 2>nul | find /i "siro.exe" >nul
if not errorlevel 1 (
    echo [*] Detected running siro.exe instance. Closing it now...
    taskkill /F /IM siro.exe /T >nul 2>&1
    ping -n 2 127.0.0.1 >nul
    echo [*] Closed successfully.
) else (
    echo [*] No running instance found.
)

echo.
echo ======================================================================
echo [2/2] Building and launching Siro (Release mode - 60 FPS optimized)...
echo ======================================================================
cargo run --bin siro --release
if errorlevel 1 (
    echo.
    echo [!] Release build or execution failed with error code %errorlevel%.
    echo Press any key to return to menu...
    pause >nul
)
goto MENU

:FAST_LAUNCH
echo.
echo ======================================================================
echo Fast Launch (Instant start without cargo recompilation)
echo ======================================================================
tasklist /fi "imagename eq siro.exe" 2>nul | find /i "siro.exe" >nul
if not errorlevel 1 (
    echo [*] Closing existing running instance...
    taskkill /F /IM siro.exe /T >nul 2>&1
    ping -n 2 127.0.0.1 >nul
)

if exist "target\release\siro.exe" (
    echo [*] Launching existing Release build: target\release\siro.exe
    start "" "target\release\siro.exe"
    goto MENU
)

if exist "target\debug\siro.exe" (
    echo [*] Launching existing Debug build: target\debug\siro.exe
    start "" "target\debug\siro.exe"
    goto MENU
)

echo [!] No pre-compiled binary found in target\release\ or target\debug\.
echo [*] Please build the app first using Option 1 or Option 2.
echo.
pause
goto MENU

:VERIFY_ALL
echo.
echo ======================================================================
echo [Step 1/2] Typecheck and Compiler Diagnostics (cargo check)
echo ======================================================================
cargo check --all-targets
if errorlevel 1 (
    echo.
    echo [!] cargo check failed. Please resolve compiler errors above.
    echo.
    pause
    goto MENU
)
echo.
echo [*] Typecheck passed cleanly.
echo.
echo ======================================================================
echo [Step 2/2] Running Complete Test Suite (cargo test)
echo ======================================================================
cargo test
if errorlevel 1 (
    echo.
    echo [!] Tests failed. Please inspect failure output above.
    echo.
    pause
    goto MENU
)
echo.
echo ======================================================================
echo [*] ALL CHECKS AND TESTS PASSED SUCCESSFULLY!
echo ======================================================================
echo.
pause
goto MENU

:KILL_APP
echo.
echo ======================================================================
echo Terminating Siro Processes
echo ======================================================================
tasklist /fi "imagename eq siro.exe" 2>nul | find /i "siro.exe" >nul
if not errorlevel 1 (
    echo [*] Terminating siro.exe...
    taskkill /F /IM siro.exe /T
    echo [*] Siro has been terminated.
) else (
    echo [*] Siro is not currently running.
)
echo.
ping -n 2 127.0.0.1 >nul
goto MENU

:CLEAN_CACHE
echo.
echo ======================================================================
echo Clean Build Cache (cargo clean)
echo ======================================================================
echo NOTE:
echo  - This only deletes temporary compiled cache in the target\ folder.
echo  - Source code, tests, assets, and saved .siro boards are 100%% safe.
echo  - The next build will recompile dependencies from scratch.
echo.
set /p "CONFIRM=Are you sure you want to clean the build cache? (Y/N): "
if /i not "!CONFIRM!"=="Y" (
    echo [*] Clean cancelled.
    ping -n 2 127.0.0.1 >nul
    goto MENU
)

echo.
echo [*] Closing any running instances to release file locks...
taskkill /F /IM siro.exe /T >nul 2>&1
ping -n 2 127.0.0.1 >nul

echo [*] Running cargo clean...
cargo clean
if errorlevel 1 (
    echo [!] cargo clean encountered an error.
) else (
    echo [*] Build cache cleaned successfully.
)
echo.
pause
goto MENU

:PACKAGE_RELEASE
echo.
echo ======================================================================
echo Packaging Release Deliverables (Portable Binary, Setup, MSI)
echo ======================================================================
tasklist /fi "imagename eq siro.exe" 2>nul | find /i "siro.exe" >nul
if not errorlevel 1 (
    echo [*] Closing running siro.exe instance...
    taskkill /F /IM siro.exe /T >nul 2>&1
    ping -n 2 127.0.0.1 >nul
)

powershell -NoProfile -ExecutionPolicy Bypass -File "scripts\package.ps1"
if errorlevel 1 (
    echo.
    echo [!] Packaging script encountered an error.
    pause
    goto MENU
)

echo.
pause
goto MENU

:EXIT_SCRIPT
echo.
echo Exiting Siro Desktop Manager.
exit /b 0
