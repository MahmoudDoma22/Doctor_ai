param([switch]$CheckOnly)
$ErrorActionPreference = "Continue"
Write-Host "Doctor AI local services prerequisite check"
$missing = @()
if (-not (Get-Command java -ErrorAction SilentlyContinue)) { $missing += "Java 21+" }
if (-not (Get-Command python -ErrorAction SilentlyContinue)) { $missing += "Python 3.10+" }
if (-not (Get-Command ollama -ErrorAction SilentlyContinue)) { $missing += "Ollama" }
if ($missing.Count -gt 0) {
  Write-Host "Missing prerequisites: $($missing -join ', ')" -ForegroundColor Yellow
  Write-Host "Install these prerequisites, then start Ollama with the gemma4:12b model."
  exit 1
}
Write-Host "Prerequisite commands found. Ensure Ollama is running and gemma4:12b is available."
if (-not $CheckOnly) {
  $root = Split-Path -Parent $MyInvocation.MyCommand.Path
  Start-Process java -ArgumentList @("-jar", (Join-Path $root "backend\doctor-ai-backend.jar")) -WorkingDirectory $root
  Write-Host "Backend launch requested. Start the media service separately after installing media requirements."
  Start-Process (Join-Path $root "desktop\DoctorAI.exe")
}
