$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$candidates = @(
  $env:GIT_BASH,
  "C:\Program Files\Git\bin\bash.exe",
  "C:\Program Files\Git\usr\bin\bash.exe",
  "C:\Program Files (x86)\Git\bin\bash.exe",
  "C:\Program Files (x86)\Git\usr\bin\bash.exe"
) | Where-Object { $_ -and (Test-Path $_) }

if (-not $candidates) {
  throw "Git Bash was not found. Set GIT_BASH or install Git for Windows."
}

& $candidates[0] (Join-Path $scriptDir "demo.sh") @args
exit $LASTEXITCODE
