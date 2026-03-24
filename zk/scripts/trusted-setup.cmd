@echo off
setlocal EnableDelayedExpansion

:: ─────────────────────────────────────────────
::  Alien Gateway — Trusted Setup (Windows)
:: ─────────────────────────────────────────────

set SCRIPT_DIR=%~dp0
set ZK_DIR=%SCRIPT_DIR%..
set PTAU_DIR=%ZK_DIR%\ptau
set BUILD_DIR=%ZK_DIR%\build

:: Power of 2 — 2^14 = 16384 constraints (covers all 3 circuits)
set POW=14

if not exist "%PTAU_DIR%" mkdir "%PTAU_DIR%"

echo.
echo ================================================
echo    Alien Gateway -- Trusted Setup
echo ================================================
echo.

:: ── Phase 1: Powers of Tau ────────────────────

set PTAU_0=%PTAU_DIR%\pot%POW%_0000.ptau
set PTAU_1=%PTAU_DIR%\pot%POW%_0001.ptau
set PTAU_FINAL=%PTAU_DIR%\pot%POW%_final.ptau

if exist "%PTAU_FINAL%" (
  echo [SKIP] Phase 1 already done -- %PTAU_FINAL% exists
) else (
  echo ^> Phase 1 -- Powers of Tau  [bn128, power=%POW%]

  snarkjs powersoftau new bn128 %POW% "%PTAU_0%" -v
  if errorlevel 1 ( echo [FAIL] powersoftau new & goto :error )
  echo   [OK] pot new done

  snarkjs powersoftau contribute "%PTAU_0%" "%PTAU_1%" ^
    --name="Alien Gateway contribution" -v
  if errorlevel 1 ( echo [FAIL] powersoftau contribute & goto :error )
  echo   [OK] pot contribute done

  snarkjs powersoftau prepare phase2 "%PTAU_1%" "%PTAU_FINAL%" -v
  if errorlevel 1 ( echo [FAIL] powersoftau prepare phase2 & goto :error )
  echo   [OK] pot prepare phase2 done
)

echo.

:: ── Phase 2: Per-circuit setup ────────────────

for %%C in (merkle_inclusion merkle_update merkle_update_proof username_merkle username_hash) do (
  echo ^> Phase 2 -- %%C

  set R1CS=%BUILD_DIR%\%%C\%%C.r1cs
  set ZKEY_0=%BUILD_DIR%\%%C\%%C_0000.zkey
  set ZKEY_FINAL=%BUILD_DIR%\%%C\%%C_final.zkey
  set VKEY=%BUILD_DIR%\%%C\verification_key.json

  if not exist "!R1CS!" (
    echo   [FAIL] !R1CS! not found -- run compile first
    goto :error
  )

  snarkjs groth16 setup "!R1CS!" "%PTAU_FINAL%" "!ZKEY_0!"
  if errorlevel 1 ( echo [FAIL] groth16 setup %%C & goto :error )
  echo   [OK] groth16 setup done

  snarkjs zkey contribute "!ZKEY_0!" "!ZKEY_FINAL!" ^
    --name="%%C contribution" -v
  if errorlevel 1 ( echo [FAIL] zkey contribute %%C & goto :error )
  echo   [OK] zkey contribute done

  snarkjs zkey export verificationkey "!ZKEY_FINAL!" "!VKEY!"
  if errorlevel 1 ( echo [FAIL] zkey export %%C & goto :error )
  echo   [OK] verification key exported
  echo        !ZKEY_FINAL!
  echo        !VKEY!
  echo.
)

echo ================================================
echo    Trusted setup complete!
echo ================================================
echo.
endlocal
exit /b 0

:error
echo.
echo ================================================
echo    Setup FAILED. See errors above.
echo ================================================
echo.
endlocal
exit /b 1