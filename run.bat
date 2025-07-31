@echo off
echo Solana Mint Address Generator
echo ==============================
echo.
echo Available commands:
echo 1. Generate pump addresses
echo 2. Generate bonk addresses  
echo 3. Generate both types
echo.
set /p choice="Enter your choice (1-3): "

if "%choice%"=="1" (
    set /p count="How many pump addresses? (default: 1): "
    if "%count%"=="" set count=1
    set /p format="Output format (json/txt, default: json): "
    if "%format%"=="" set format=json
    set /p output="Output filename (optional, press Enter for auto-generated): "
    if "%output%"=="" (
        echo 🚀 Running with maximum performance (all CPU cores)...
        cargo run --release -- pump --count %count% --format %format%
    ) else (
        echo 🚀 Running with maximum performance (all CPU cores)...
        cargo run --release -- pump --count %count% --format %format% --output %output%
    )
) else if "%choice%"=="2" (
    set /p count="How many bonk addresses? (default: 1): "
    if "%count%"=="" set count=1
    set /p format="Output format (json/txt, default: json): "
    if "%format%"=="" set format=json
    set /p output="Output filename (optional, press Enter for auto-generated): "
    if "%output%"=="" (
        echo 🚀 Running with maximum performance (all CPU cores)...
        cargo run --release -- bonk --count %count% --format %format%
    ) else (
        echo 🚀 Running with maximum performance (all CPU cores)...
        cargo run --release -- bonk --count %count% --format %format% --output %output%
    )
) else if "%choice%"=="3" (
    set /p count="How many addresses of each type? (default: 1): "
    if "%count%"=="" set count=1
    set /p format="Output format (json/txt, default: json): "
    if "%format%"=="" set format=json
    set /p output="Output filename prefix (optional, press Enter for auto-generated): "
    if "%output%"=="" (
        echo 🚀 Running with maximum performance (all CPU cores)...
        cargo run --release -- both --count %count% --format %format%
    ) else (
        echo 🚀 Running with maximum performance (all CPU cores)...
        cargo run --release -- both --count %count% --format %format% --output %output%
    )
) else (
    echo Invalid choice!
    pause
    exit /b 1
)

echo.
echo Generation complete!
pause 