@echo off
setlocal enabledelayedexpansion

:: It will default to searching for love
:: but you can override the default by
:: setting the `LOVE` env var to another name
:: ex: 
:: ```
:: set LOVE="C:\Program Files\LOVE\love.exe"
:: ``

:: Check if LOVE environment variable is set
if not "%LOVE%"=="" (
    if exist "%LOVE%" (
        set LOVE=%LOVE%
        goto execute
    )
)

:: Search through PATH
for %%I in (love.exe) do set love_path=%%~$PATH:I
if exist "!love_path!" (
    set LOVE=!love_path!
    goto execute
)

:: Check common install locations
set check_folders="%ProgramFiles%\LOVE\love.exe" "%ProgramFiles(x86)%\LOVE\love.exe" "%APPDATA%\LOVE\love.exe" "%LOCALAPPDATA%\LOVE\love.exe"
for %%I in (%check_folders%) do (
    if exist "%%~I" (
        set LOVE=%%~I
        goto execute
    )
)

:: Error if not found
echo Error: love.exe not found in PATH, Program Files, or AppData
echo Install LÖVE or set LOVE environment variable to your executable
exit /b 1

:: make sure to preserve current working directory
:execute
pushd lua
"%LOVE%" . 
popd

endlocal