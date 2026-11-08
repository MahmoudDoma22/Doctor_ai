$ErrorActionPreference = "Stop"
$Model = "gemma4:e4b"
$OllamaUrl = "http://127.0.0.1:11434"
Write-Host ""
Write-Host "Doctor AI - Gemma 4 E4B setup" -ForegroundColor Cyan
Write-Host "This download is several gigabytes. Keep the PC connected to the internet."
Write-Host ""
$ollamaCommand = Get-Command "ollama.exe" -ErrorAction SilentlyContinue
$ollama = if ($ollamaCommand) { $ollamaCommand.Source } else { $null }
if (-not $ollama) {
  $candidates = @(
    (Join-Path $env:LOCALAPPDATA "Programs\Ollama\ollama.exe"),
    (Join-Path $env:ProgramFiles "Ollama\ollama.exe")
  ) | Where-Object { $_ -and (Test-Path $_) }
  if ($candidates.Count -gt 0) { $ollama = $candidates[0] }
}
if (-not $ollama) {
  Write-Host "Ollama was not found. Install it from https://ollama.com/download, then run this setup again." -ForegroundColor Red
  exit 2
}
$ready = $false
try {
  Invoke-RestMethod "$OllamaUrl/api/tags" -TimeoutSec 3 | Out-Null
  $ready = $true
} catch {
  Write-Host "Starting the local Ollama service..."
  Start-Process -FilePath $ollama -ArgumentList "serve" -WindowStyle Hidden
  for ($i = 0; $i -lt 40; $i++) {
    Start-Sleep -Seconds 2
    try {
      Invoke-RestMethod "$OllamaUrl/api/tags" -TimeoutSec 2 | Out-Null
      $ready = $true
      break
    } catch {}
  }
}
if (-not $ready) {
  Write-Host "Ollama did not become ready at $OllamaUrl. Open Ollama and retry." -ForegroundColor Red
  exit 3
}
Write-Host "Downloading $Model. This may take a while..." -ForegroundColor Yellow
& $ollama pull $Model
if ($LASTEXITCODE -ne 0) {
  Write-Host "Model download failed (exit code $LASTEXITCODE). Check internet/disk space and retry from this script." -ForegroundColor Red
  exit $LASTEXITCODE
}
$models = & $ollama list
if ($LASTEXITCODE -ne 0 -or -not (($models -join [Environment]::NewLine) -match [regex]::Escape($Model))) {
  Write-Host "The download command ended, but $Model was not found in 'ollama list'." -ForegroundColor Red
  exit 4
}
Write-Host ""
Write-Host "$Model is installed and ready in Ollama." -ForegroundColor Green
exit 0
