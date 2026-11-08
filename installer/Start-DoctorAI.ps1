$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $MyInvocation.MyCommand.Path
$java = Join-Path $root "runtime\java\bin\java.exe"
$jar = Join-Path $root "backend\doctor-ai-backend.jar"
$app = Join-Path $root "desktop\DoctorAI.exe"
if (-not (Test-Path $java)) { throw "Bundled Java runtime is missing." }
if (-not (Test-Path $jar)) { throw "Doctor AI backend is missing." }
if (-not (Test-Path $app)) { throw "Doctor AI desktop application is missing." }
try { Invoke-RestMethod "http://127.0.0.1:8787/api/health" -TimeoutSec 2 | Out-Null }
catch {
 Start-Process -FilePath $java -ArgumentList @("-jar", ('"' + $jar + '"')) -WorkingDirectory $root -WindowStyle Hidden
 $backendReady=$false
 for($i=0;$i -lt 40;$i++){Start-Sleep -Milliseconds 750;try{$h=Invoke-RestMethod "http://127.0.0.1:8787/api/health" -TimeoutSec 2;if($h.backend -eq $true){$backendReady=$true;break}}catch{}}
 if(-not $backendReady){throw "Doctor AI backend did not start. Check the installation and runtime files."}
}
Start-Process -FilePath $app -WorkingDirectory (Split-Path -Parent $app)
