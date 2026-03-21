CHOICE /C YN /M "Install deno runtime? [Y/N]"
IF %ERRORLEVEL% EQU 1 (
    powershell -ExecutionPolicy Bypass -File install_deno.ps1
)

CHOICE /C YN /M "Install kokoro-tts runtime? [Y/N]"
IF %ERRORLEVEL% EQU 1 (
    cd .\kokoro
    deno install --frozen --node-modules-dir=manual
    cd ..
)
CHOICE /C YN /M "Download kokoro-tts models? [Y/N]"
IF %ERRORLEVEL% EQU 1 (
    cd .\kokoro
    deno install_models.js
    cd ..
)

CHOICE /C YN /M "Download bergamot models for local translation? [Y/N]"
IF %ERRORLEVEL% EQU 1 (
    cd .\bergamot
    deno install_models.js
    cd ..
)

CHOICE /C YN /M "Clear deno cache? [Y/N]"
IF %ERRORLEVEL% EQU 1 (
    deno clean
)

pause
