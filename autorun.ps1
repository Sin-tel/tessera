#!/usr/bin/env pwsh

# It will default to searching for love
# but you can override the default by
# setting the `LOVE` env var to another name
# ex: 
# ```
# $env:LOVE = "C:\Program Files\LOVE\love.exe"
# ``

$love = $env:LOVE, (($env:PATH -split ';') + 
    @("${env:ProgramFiles}\LOVE", "${env:ProgramFiles(x86)}\LOVE", 
      "${env:APPDATA}\LOVE", "${env:LocalAppData}\LOVE") |% { 
          Join-Path $_ love.exe 
      }) | Where-Object { Test-Path $_ } | Select-Object -First 1

if (!$love) { throw "love.exe not found in: PATH, Program Files, AppData. Install or set `$env:LOVE" }

# make sure to preserve current working directory
Push-Location lua
try { & $love . } finally { Pop-Location }