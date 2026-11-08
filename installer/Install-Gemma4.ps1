$ErrorActionPreference = "Stop"
$Model = "gemma4:e4b"
$OllamaUrl = "http://127.0.0.1:11434"
$LogPath = Join-Path $env:TEMP "DoctorAI-Gemma4-Setup.log"
try {
  "Doctor AI Gemma 4 setup started: $(Get-Date -Format o)" | Set-Content -Path $LogPath
  $ollamaCommand = Get-Command "ollama.exe" -ErrorAction SilentlyContinue
  $ollama = if ($ollamaCommand) { $ollamaCommand.Source } else { $null }
  if (-not $ollama) {
    $candidates = @(
      (Join-Path $env:LOCALAPPDATA "Programs\Ollama\ollama.exe"),
      (Join-Path $env:ProgramFiles "Ollama\ollama.exe")
    ) | Where-Object { $_ -and (Test-Path $_) }
    if ($candidates.Count -gt 0) { $ollama = $candidates[0] }
  }
  if (-not $ollama) { throw "Ollama was not found. Install it from https://ollama.com/download and run this setup again." }

  $ready = $false
  try {
    Invoke-RestMethod "$OllamaUrl/api/tags" -TimeoutSec 3 | Out-Null
    $ready = $true
  } catch {
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
  if (-not $ready) { throw "Ollama did not become ready at $OllamaUrl. Open Ollama and retry." }

  & $ollama pull $Model 2>&1 | Out-File -FilePath $LogPath -Append
  if ($LASTEXITCODE -ne 0) { throw "ollama pull failed with exit code $LASTEXITCODE. See $LogPath" }

  $models = & $ollama list
  if ($LASTEXITCODE -ne 0 -or -not (($models -join [Environment]::NewLine) -match [regex]::Escape($Model))) {
    throw "The download ended, but $Model was not found in ollama list. See $LogPath"
  }
  "SUCCESS: $Model installed at $(Get-Date -Format o)" | Add-Content -Path $LogPath
  exit 0
}
catch {
  ("ERROR: " + $_.Exception.Message) | Add-Content -Path $LogPath
  exit 10
}
