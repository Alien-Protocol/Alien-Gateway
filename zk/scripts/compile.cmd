@echo off
setlocal EnableDelayedExpansion

:: ─────────────────────────────────────────────
::  Alien Gateway — ZK Circuit Compiler (Windows)
:: ─────────────────────────────────────────────

set SCRIPT_DIR=%~dp0
set ZK_DIR=%SCRIPT_DIR%..
set CIRCUITS_DIR=%ZK_DIR%\circuits
set BUILD_DIR=%ZK_DIR%\build
set NODE_MODULES=%ZK_DIR%\node_modules

echo.
echo ================================================
echo    Alien Gateway -- ZK Circuit Compiler
echo ================================================
echo.

:: ── Compile each circuit ──────────────────────

call :compile_circuit "merkle_inclusion" "merkle\merkle_inclusion.circom"
if errorlevel 1 goto :error

call :compile_circuit "merkle_update" "merkle_update.circom"
if errorlevel 1 goto :error

call :compile_circuit "merkle_update_proof" "merkle\merkle_update_proof.circom"
if errorlevel 1 goto :error

call :compile_circuit "username_merkle" "username_merkle.circom"
if errorlevel 1 goto :error

call :compile_circuit "username_hash" "username_hash.circom"
if errorlevel 1 goto :error

echo ================================================
echo    All circuits compiled successfully!
echo ================================================
echo.
endlocal
exit /b 0

:: ── Subroutine ───────────────────────────────

:compile_circuit
set NAME=%~1
set CIRCOM_PATH=%~2

echo ^> Compiling: %NAME%

set OUT_DIR=%BUILD_DIR%\%NAME%
set WASM_DIR=%OUT_DIR%\wasm

if not exist "%OUT_DIR%"  mkdir "%OUT_DIR%"
if not exist "%WASM_DIR%" mkdir "%WASM_DIR%"

:: r1cs + sym
circom %CIRCUITS_DIR%\%CIRCOM_PATH% ^
  --r1cs --sym ^
  -o %OUT_DIR% ^
  -l %NODE_MODULES%
if errorlevel 1 (
  echo   [FAIL] %NAME% -- r1cs/sym compilation failed
  exit /b 1
)

:: wasm into wasm\ subfolder
circom %CIRCUITS_DIR%\%CIRCOM_PATH% ^
  --wasm ^
  -o %WASM_DIR% ^
  -l %NODE_MODULES%
if errorlevel 1 (
  echo   [FAIL] %NAME% -- wasm compilation failed
  exit /b 1
)

echo   [OK]   %NAME% compiled
echo          %OUT_DIR%\%NAME%.r1cs
echo          %OUT_DIR%\%NAME%.sym
echo          %WASM_DIR%\%NAME%_js\%NAME%.wasm
echo.
exit /b 0

:error
echo.
echo ================================================
echo    Compilation FAILED. See errors above.
echo ================================================
echo.
endlocal
exit /b 1