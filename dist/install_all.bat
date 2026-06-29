CHOICE /C YN /M "Install deno runtime? [Y/N]"
IF %ERRORLEVEL% EQU 1 (
    powershell -ExecutionPolicy Bypass -Command "irm https://deno.land/install.ps1 | iex"
    echo Please restart your terminal to apply PATH changes.
)

CHOICE /C YN /M "Install kokoro-tts runtime? [Y/N]"
IF %ERRORLEVEL% EQU 1 (
    cd .\extensions\kokoro
    deno install --frozen --node-modules-dir=manual
    cd ..
)
CHOICE /C YN /M "Download kokoro-tts models? [Y/N]"
IF %ERRORLEVEL% EQU 1 (
    cd .\extensions\kokoro
    deno install_models.js
    cd ..
)

CHOICE /C YN /M "Download bergamot models for local translation? [Y/N]"
IF %ERRORLEVEL% EQU 1 (
    cd .\extensions\tr_bergamot
    deno install_models.js
    cd ..\..
)

CHOICE /C YN /M "Clear deno cache? [Y/N]"
IF %ERRORLEVEL% EQU 1 (
    deno clean
)

pause
