@echo off
title Solana Mint Generator - Auto Mode
color 0A

echo ===============================================
echo    Solana Mint Generator - Continuous Mode
echo ===============================================
echo.
echo Configuration:
echo - Address type: PUMP
echo - Count per batch: 1000
echo - Format: JSON
echo - Auto-restart: ENABLED
echo.
echo Press Ctrl+C to stop
echo.

:start
echo [%date% %time%] Starting new batch...
cargo run --release -- pump --count 1000

echo.
echo [%date% %time%] Batch complete. Waiting 30 seconds before next batch...
timeout /t 30 /nobreak >nul

goto start 