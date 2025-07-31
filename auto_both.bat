@echo off
title Solana Mint Generator - Both Types Auto Mode
color 0B

echo ===============================================
echo  Solana Mint Generator - Both Types Auto Mode
echo ===============================================
echo.
echo Configuration:
echo - Address types: PUMP + BONK
echo - Count per type: 500
echo - Format: JSON
echo - Auto-restart: ENABLED
echo.
echo Press Ctrl+C to stop
echo.

:start
echo [%date% %time%] Starting new batch (both types)...
cargo run --release -- both --count 500

echo.
echo [%date% %time%] Batch complete. Waiting 60 seconds before next batch...
timeout /t 60 /nobreak >nul

goto start 