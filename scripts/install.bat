@echo off
REM RiceCoder Installation Script for Windows (CMD)
REM
REM This script compiles and installs ricecoder from source on Windows.
REM Supports: Windows 10/11 (x86_64, aarch64)
REM
REM Usage:
REM   install.bat [OPTIONS]
REM
REM Options:
REM   --prefix PATH       Installation prefix (default: %LOCALAPPDATA%\ricecoder)
REM   --release           Build in release mode (default)
REM   --debug             Build in debug mode
REM   --help              Show this help message
REM
REM Examples:
REM   install.bat
REM   install.bat --prefix "C:\Program Files\ricecoder"
REM   install.bat --debug
REM
REM Requirements:
REM   - Rust (https://rustup.rs/)
REM   - Git (https://git-scm.com/)
REM   - Visual Studio Build Tools or MSVC compiler
REM

setlocal enabledelayedexpansion

REM Configuration
set "SCRIPT_DIR=%~dp0"
set "PROJECT_ROOT=%SCRIPT_DIR%.."
set "PREFIX=%LOCALAPPDATA%\ricecoder"
set "BUILD_MODE=release"
set "VERBOSE=0"

REM Parse command line arguments
:parse_args
if "%~1"=="" goto args_done
if "%~1"=="--prefix" (
    set "PREFIX=%~2"
    shift
    shift
    goto parse_args
)
if "%~1"=="--release" (
    set "BUILD_MODE=release"
    shift
    goto parse_args
)
if "%~1"=="--debug" (
    set "BUILD_MODE=debug"
    shift
    goto parse_args
)
if "%~1"=="--verbose" (
    set "VERBOSE=1"
    shift
    goto parse_args
)
if "%~1"=="--help" (
    call :show_help
    exit /b 0
)
shift
goto parse_args

:args_done

REM Detect architecture
if "%PROCESSOR_ARCHITECTURE%"=="AMD64" (
    set "ARCH=x86_64"
) else if "%PROCESSOR_ARCHITECTURE%"=="ARM64" (
    set "ARCH=aarch64"
) else (
    set "ARCH=%PROCESSOR_ARCHITECTURE%"
)

echo.
echo ========================================================
echo        RiceCoder Installation Script
echo ========================================================
echo.
echo System Information:
echo   OS:           Windows
echo   Architecture: %ARCH%
echo   Project Root: %PROJECT_ROOT%
echo.

REM Check prerequisites
call :check_prerequisites
if errorlevel 1 exit /b 1

REM Update Rust
call :update_rust
if errorlevel 1 exit /b 1

REM Build ricecoder
call :build_ricecoder
if errorlevel 1 exit /b 1

REM Install binaries
call :install_binaries
if errorlevel 1 exit /b 1

REM Install configuration
call :install_config
if errorlevel 1 exit /b 1

REM Install documentation
call :install_docs
if errorlevel 1 exit /b 1

REM Verify installation
call :verify_installation
if errorlevel 1 exit /b 1

REM Print summary
call :print_summary

exit /b 0

REM ========================================================
REM Functions
REM ========================================================

:show_help
echo RiceCoder Installation Script for Windows
echo.
echo Usage:
echo   install.bat [OPTIONS]
echo.
echo Options:
echo   --prefix PATH       Installation prefix (default: %%LOCALAPPDATA%%\ricecoder)
echo   --release           Build in release mode (default)
echo   --debug             Build in debug mode
echo   --verbose           Show verbose output
echo   --help              Show this help message
echo.
echo Examples:
echo   install.bat
echo   install.bat --prefix "C:\Program Files\ricecoder"
echo   install.bat --debug
echo.
echo Requirements:
echo   - Rust (https://rustup.rs/)
echo   - Git (https://git-scm.com/)
echo   - Visual Studio Build Tools or MSVC compiler
echo.
exit /b 0

:check_prerequisites
echo [INFO] Checking prerequisites...

REM Check Rust
where rustc >nul 2>&1
if errorlevel 1 (
    echo [ERROR] Rust is not installed
    echo Install Rust from: https://rustup.rs/
    exit /b 1
)

for /f "tokens=*" %%i in ('rustc --version') do set "RUST_VERSION=%%i"
echo [OK] Found: %RUST_VERSION%

REM Check Cargo
where cargo >nul 2>&1
if errorlevel 1 (
    echo [ERROR] Cargo is not installed
    exit /b 1
)

REM Check Git
where git >nul 2>&1
if errorlevel 1 (
    echo [ERROR] Git is not installed
    exit /b 1
)

REM Check MSVC
where cl.exe >nul 2>&1
if errorlevel 1 (
    echo [WARNING] MSVC compiler not found in PATH
    echo Make sure Visual Studio Build Tools or MSVC is installed
)

echo [OK] All prerequisites met
exit /b 0

:update_rust
echo [INFO] Updating Rust toolchain...
call rustup update
if errorlevel 1 (
    echo [ERROR] Failed to update Rust
    exit /b 1
)
echo [OK] Rust toolchain updated
exit /b 0

:build_ricecoder
echo [INFO] Building ricecoder (%BUILD_MODE% mode)...

cd /d "%PROJECT_ROOT%"

set "CARGO_ARGS="
if "%BUILD_MODE%"=="release" (
    set "CARGO_ARGS=--release"
)
if "%VERBOSE%"=="1" (
    set "CARGO_ARGS=%CARGO_ARGS% --verbose"
)

call cargo build %CARGO_ARGS%
if errorlevel 1 (
    echo [ERROR] Build failed
    exit /b 1
)

echo [OK] Build completed successfully
exit /b 0

:install_binaries
echo [INFO] Installing binaries to %PREFIX%\bin...

if not exist "%PREFIX%\bin" mkdir "%PREFIX%\bin"

if "%BUILD_MODE%"=="release" (
    set "BUILD_DIR=%PROJECT_ROOT%\target\release"
) else (
    set "BUILD_DIR=%PROJECT_ROOT%\target\debug"
)

setlocal enabledelayedexpansion
for /f "delims=" %%f in ('dir /b "%BUILD_DIR%\*.exe" 2^>nul') do (
    set "BINARY=%%f"
    if not "!BINARY:~0,3!"=="lib" (
        echo [INFO] Installing !BINARY!...
        copy /y "%BUILD_DIR%\!BINARY!" "%PREFIX%\bin\!BINARY!" >nul
        if errorlevel 1 (
            echo [ERROR] Failed to install !BINARY!
            exit /b 1
        )
        echo [OK] Installed !BINARY!
    )
)
endlocal

exit /b 0

:install_config
echo [INFO] Installing configuration files...

if not exist "%PREFIX%\etc\ricecoder" mkdir "%PREFIX%\etc\ricecoder"

if exist "%PROJECT_ROOT%\config" (
    xcopy /e /i /y "%PROJECT_ROOT%\config\*" "%PREFIX%\etc\ricecoder\" >nul 2>&1
    echo [OK] Configuration files installed to %PREFIX%\etc\ricecoder
)

exit /b 0

:install_docs
echo [INFO] Installing documentation...

if not exist "%PREFIX%\share\doc\ricecoder" mkdir "%PREFIX%\share\doc\ricecoder"

if exist "%PROJECT_ROOT%\README.md" (
    copy /y "%PROJECT_ROOT%\README.md" "%PREFIX%\share\doc\ricecoder\" >nul
)

if exist "%PROJECT_ROOT%\LICENSE.md" (
    copy /y "%PROJECT_ROOT%\LICENSE.md" "%PREFIX%\share\doc\ricecoder\" >nul
)

echo [OK] Documentation installed to %PREFIX%\share\doc\ricecoder

exit /b 0

:verify_installation
echo [INFO] Verifying installation...

if not exist "%PREFIX%\bin" (
    echo [ERROR] Installation directory not found: %PREFIX%\bin
    exit /b 1
)

setlocal enabledelayedexpansion
set "COUNT=0"
for /f "delims=" %%f in ('dir /b "%PREFIX%\bin\ricecoder*.exe" 2^>nul') do (
    echo [OK] Found: %%f
    set /a COUNT+=1
)

if !COUNT! equ 0 (
    echo [WARNING] No ricecoder binaries found in %PREFIX%\bin
    exit /b 1
)

echo [OK] Installation verified (!COUNT! binaries)
endlocal

exit /b 0

:print_summary
echo.
echo ========================================================
echo        RiceCoder Installation Complete!
echo ========================================================
echo.
echo Installation Details:
echo   OS:              Windows
echo   Architecture:    %ARCH%
echo   Build Mode:      %BUILD_MODE%
echo   Install Prefix:  %PREFIX%
echo   Binaries:        %PREFIX%\bin
echo   Config:          %PREFIX%\etc\ricecoder
echo   Documentation:   %PREFIX%\share\doc\ricecoder
echo.
echo Next Steps:
echo   1. Add to PATH: set PATH=%%PATH%%;%PREFIX%\bin
echo   2. Verify:      ricecoder --version
echo   3. Get help:    ricecoder --help
echo.

exit /b 0
