param([switch]$CheckOnly)
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $MyInvocation.MyCommand.Path
$jar = Join-Path $root "backend\doctor-ai-backend.jar"
$desktop = Join-Path $root "desktop\DoctorAI.exe"
$baseUrl = "http://127.0.0.1:8787"
$logDir = Join-Path $env:LOCALAPPDATA "DoctorAI\logs"
$logFile = Join-Path $logDir "backend.log"
function Show-StartupError([string]$message) {
  Add-Type -AssemblyName PresentationFramework
  [System.Windows.MessageBox]::Show($message, "Doctor AI - Startup", "OK", "Warning") | Out-Null
}
if (-not (Test-Path $jar)) {
  Show-StartupError "The Doctor AI Java backend is missing: $jar. Please reinstall Doctor AI."
  exit 1
}
if (-not (Test-Path $desktop)) {
  Show-StartupError "The Doctor AI desktop application is missing: $desktop. Please reinstall Doctor AI."
  exit 1
}
$ready = $false
try {
  $response = Invoke-WebRequest -Uri "$baseUrl/api/health" -UseBasicParsing -TimeoutSec 2
  $ready = ($response.StatusCode -eq 200)
} catch {}
if (-not $ready) {
  $java = Get-Command java -ErrorAction SilentlyContinue
  if (-not $java) {
    Show-StartupError "Java 21 or later was not found. Install a Java 21+ runtime, then open Doctor AI again."
    exit 1
  }
  New-Item -ItemType Directory -Force -Path $logDir | Out-Null
  try {
    $javaArgs = '-jar "' + $jar + '"'
    Start-Process -FilePath $java.Source -ArgumentList $javaArgs -WorkingDirectory $root -WindowStyle Hidden -RedirectStandardOutput $logFile -RedirectStandardError (Join-Path $logDir "backend-error.log") | Out-Null
  } catch {
    Show-StartupError "Could not start the local Java backend: $($_.Exception.Message)"
    exit 1
  }
  for ($i = 0; $i -lt 45; $i++) {
    Start-Sleep -Seconds 1
    try {
      $response = Invoke-WebRequest -Uri "$baseUrl/api/health" -UseBasicParsing -TimeoutSec 2
      if ($response.StatusCode -eq 200) { $ready = $true; break }
    } catch {}
  }
}
if (-not $ready) {
  $detail = ""
  $errorLog = Join-Path $logDir "backend-error.log"
  if (Test-Path $errorLog) {
    $detail = (Get-Content $errorLog -Tail 12 -ErrorAction SilentlyContinue) -join [Environment]::NewLine
  }
  Show-StartupError "The Java backend did not start on 127.0.0.1:8787 within 45 seconds. Check that port 8787 is free and Java 21+ is installed. $detail"
  exit 1
}
if (-not $CheckOnly) {
  Start-Process -FilePath $desktop -WorkingDirectory (Split-Path -Parent $desktop)
}
