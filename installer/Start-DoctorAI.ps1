param([switch]$CheckOnly)
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $MyInvocation.MyCommand.Path
$jar = Join-Path $root "backend\doctor-ai-backend.jar"
$desktop = Join-Path $root "desktop\DoctorAI.exe"
$bundledJava = Join-Path $root "runtime\bin\java.exe"
$baseUrl = "http://127.0.0.1:8787"
$ollamaUrl = "http://127.0.0.1:11434"
$logDir = Join-Path $env:LOCALAPPDATA "DoctorAI\logs"
$logFile = Join-Path $logDir "backend.log"
function Show-StartupError([string]$message) {
  Add-Type -AssemblyName PresentationFramework
  [System.Windows.MessageBox]::Show($message, "Doctor AI - Startup", "OK", "Warning") | Out-Null
}
function Test-Http([string]$url) {
  try { Invoke-RestMethod -Uri $url -TimeoutSec 2 } catch { $null }
}
if (-not (Test-Path $jar) -or -not (Test-Path $desktop)) {
  Show-StartupError "Doctor AI installation files are missing. Please reinstall the application."
  exit 1
}
$ollamaCommand = Get-Command ollama -ErrorAction SilentlyContinue
$ollamaExe = if ($ollamaCommand) { $ollamaCommand.Source } else { $null }
if (-not $ollamaExe) {
  $candidates = @(
    (Join-Path $env:LOCALAPPDATA "Programs\Ollama\ollama.exe"),
    (Join-Path $env:ProgramFiles "Ollama\ollama.exe"),
    (Join-Path $env:LOCALAPPDATA "Ollama\ollama.exe")
  )
  $ollamaExe = $candidates | Where-Object { $_ -and (Test-Path $_) } | Select-Object -First 1
}
$ollamaTags = Test-Http "$ollamaUrl/api/tags"
if (-not $ollamaTags -and $ollamaExe) {
  try { Start-Process -FilePath $ollamaExe -ArgumentList "serve" -WindowStyle Hidden | Out-Null } catch {}
  for ($i = 0; $i -lt 30 -and -not $ollamaTags; $i++) {
    Start-Sleep -Seconds 1
    $ollamaTags = Test-Http "$ollamaUrl/api/tags"
  }
}
if ($ollamaTags -and $ollamaTags.models) {
  $gemma = $ollamaTags.models | Where-Object { $_.name -match "(?i)^gemma" } | Select-Object -First 1
  if ($gemma) { $env:OLLAMA_MODEL = $gemma.name }
  else {
    $available = ($ollamaTags.models | ForEach-Object { $_.name }) -join ", "
    Show-StartupError ("Ollama is running, but no Gemma model was found. Available models: " + $available + [Environment]::NewLine + "Install a Gemma model with: ollama pull gemma4:12b")
    exit 1
  }
} else {
  Show-StartupError "Ollama could not be detected or started. Start Ollama and make sure your Gemma model is installed, then open Doctor AI again."
  exit 1
}
if (Test-Path $bundledJava) { $javaExe = $bundledJava }
else {
  $javaCommand = Get-Command java -ErrorAction SilentlyContinue
  $javaExe = if ($javaCommand) { $javaCommand.Source } else { $null }
}
if (-not $javaExe) {
  Show-StartupError "The bundled Java runtime is missing. Please reinstall Doctor AI."
  exit 1
}
New-Item -ItemType Directory -Force -Path $logDir | Out-Null
$ready = [bool](Test-Http "$baseUrl/api/health")
if (-not $ready) {
  try {
    $javaArgs = '-jar "' + $jar + '"'
    Start-Process -FilePath $javaExe -ArgumentList $javaArgs -WorkingDirectory $root -WindowStyle Hidden -RedirectStandardOutput $logFile -RedirectStandardError (Join-Path $logDir "backend-error.log") | Out-Null
  } catch {
    Show-StartupError "Could not start the local Java backend: $($_.Exception.Message)"
    exit 1
  }
  for ($i = 0; $i -lt 60; $i++) {
    Start-Sleep -Seconds 1
    $health = Test-Http "$baseUrl/api/health"
    if ($health -and $health.ok -eq $true) { $ready = $true; break }
  }
}
if (-not $ready) {
  $detail = ""
  $errorLog = Join-Path $logDir "backend-error.log"
  if (Test-Path $errorLog) { $detail = (Get-Content $errorLog -Tail 15 -ErrorAction SilentlyContinue) -join [Environment]::NewLine }
  Show-StartupError ("Doctor AI backend did not become ready on 127.0.0.1:8787. Check logs in " + $logDir + ". " + $detail)
  exit 1
}
if (-not $CheckOnly) { Start-Process -FilePath $desktop -WorkingDirectory (Split-Path -Parent $desktop) }
