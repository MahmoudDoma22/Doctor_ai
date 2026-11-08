param([Parameter(Mandatory=$true)][string]$Aria2Path,[switch]$Vision)
$ErrorActionPreference="Stop"
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
$ModelDir=Join-Path $env:USERPROFILE ".doctor-ai\models";$Log=Join-Path $env:TEMP "DoctorAI-Model-Download.log";$script:Cancelled=$false
$form=New-Object Windows.Forms.Form;$form.Text="Doctor AI — Offline model setup";$form.Size=New-Object Drawing.Size(560,190);$form.StartPosition="CenterScreen";$form.FormBorderStyle="FixedDialog";$form.MaximizeBox=$false;$form.MinimizeBox=$false
$label=New-Object Windows.Forms.Label;$label.Location=New-Object Drawing.Point(18,18);$label.Size=New-Object Drawing.Size(510,48);$label.Text="Checking this PC and preparing the recommended offline model...";$form.Controls.Add($label)
$bar=New-Object Windows.Forms.ProgressBar;$bar.Location=New-Object Drawing.Point(18,76);$bar.Size=New-Object Drawing.Size(510,22);$bar.Style="Marquee";$form.Controls.Add($bar)
$cancel=New-Object Windows.Forms.Button;$cancel.Location=New-Object Drawing.Point(428,112);$cancel.Size=New-Object Drawing.Size(100,28);$cancel.Text="Cancel";$cancel.Add_Click({$script:Cancelled=$true;$label.Text="Cancelling safely; partial downloads will be kept for resume."});$form.Controls.Add($cancel);$form.Show();[Windows.Forms.Application]::DoEvents()
function Download-Aria([string]$url,[string]$name,[string]$description,[long]$minimum){
 if($script:Cancelled){throw "Download cancelled."};$target=Join-Path $ModelDir $name;if((Test-Path $target)-and(Get-Item $target).Length -ge $minimum){return}
 $out=Join-Path $env:TEMP "DoctorAI-aria2-out.txt";$err=Join-Path $env:TEMP "DoctorAI-aria2-err.txt";Remove-Item $out,$err -Force -ErrorAction SilentlyContinue
 $args=@("--console-log-level=warn","--summary-interval=1","--max-connection-per-server=8","--split=8","--min-split-size=1M","--max-tries=12","--retry-wait=5","--timeout=60","--connect-timeout=20","--continue=true","--allow-overwrite=true","--auto-file-renaming=false","--file-allocation=none","--dir=$ModelDir","--out=$name",$url)
 $quoted=foreach($a in $args){if($a -match '\s'){'"'+$a.Replace('"','\"')+'"'}else{$a}}
 $p=Start-Process -FilePath $Aria2Path -ArgumentList ($quoted -join " ") -PassThru -WindowStyle Hidden -RedirectStandardOutput $out -RedirectStandardError $err
 while(-not $p.HasExited){
  [Windows.Forms.Application]::DoEvents();if($script:Cancelled){try{$p.Kill()}catch{};throw "Download cancelled; partial file retained for resume."}
  $status="";if(Test-Path $out){$tail=(Get-Content $out -Tail 8 -ErrorAction SilentlyContinue)-join " ";if($tail -match '\((\d+)%\).*?DL:([^\s\]]+).*?ETA:([^\s\]]+)'){$status=" $($Matches[1])% · $($Matches[2]) · ETA $($Matches[3])"}}
  $label.Text=$description+[Environment]::NewLine+$status;Start-Sleep -Milliseconds 400;$p.Refresh()
 }
 if($p.ExitCode -ne 0){$detail=if(Test-Path $err){Get-Content $err -Raw -ErrorAction SilentlyContinue}else{""};throw "$description failed (aria2 exit $($p.ExitCode)). Retry setup; partial data is retained. $detail"}
 if(-not(Test-Path $target)-or(Get-Item $target).Length -lt $minimum){throw "$description is incomplete. Retry setup."}
}
try{
 if(-not(Test-Path $Aria2Path)){throw "Bundled aria2 downloader is missing."}
 $sys=Get-CimInstance Win32_ComputerSystem;$ram=[math]::Round($sys.TotalPhysicalMemory/1GB);$cpu=Get-CimInstance Win32_Processor|Select-Object -First 1;$gpu=(Get-CimInstance Win32_VideoController|%{$_.Name})-join ";"
 $drive=$env:SystemDrive.Substring(0,1);$free=[math]::Round((Get-PSDrive -Name $drive).Free/1GB)
 if($ram -lt 8){throw "This PC has about $ram GB RAM. Offline AI needs at least 8 GB. You can install the desktop without the model."}
 if($ram -ge 16){
  $model="gemma-4-E4B-it-Q4_0.gguf";$url="https://huggingface.co/ggml-org/gemma-4-E4B-it-GGUF/resolve/main/gemma-4-E4B-it-Q4_0.gguf";$proj="mmproj-gemma-4-E4B-it-BF16.gguf";$projUrl="https://huggingface.co/ggml-org/gemma-4-E4B-it-GGUF/resolve/main/mmproj-gemma-4-E4B-it-BF16.gguf";$min=4GB;$recommended="Gemma 4 E4B Q4_0";$need=if($Vision){7}else{5}
 }else{
  $model="gemma-4-E2B-it-Q4_0.gguf";$url="https://huggingface.co/ggml-org/gemma-4-E2B-it-GGUF/resolve/main/gemma-4-E2B-it-Q4_0.gguf";$proj="mmproj-gemma-4-E2B-it-BF16.gguf";$projUrl="https://huggingface.co/ggml-org/gemma-4-E2B-it-GGUF/resolve/main/mmproj-gemma-4-E2B-it-BF16.gguf";$min=2GB;$recommended="Gemma 4 E2B Q4_0";$need=if($Vision){5}else{3}
 }
 if($free -lt $need){throw "Only $free GB free on $env:SystemDrive. Free at least $need GB for $recommended and selected modules."}
 New-Item -ItemType Directory -Force $ModelDir|Out-Null
 [pscustomobject]@{ramGB=$ram;cpu=$cpu.Name;logicalProcessors=$env:NUMBER_OF_PROCESSORS;gpu=$gpu;freeDiskGB=$free;recommendedModel=$recommended;visionEnabled=[bool]$Vision;detectedAt=(Get-Date).ToString("o")}|ConvertTo-Json|Set-Content (Join-Path $ModelDir "hardware-profile.json")
 "Recommended: $recommended | RAM: $ram GB | CPU: $($cpu.Name) | GPU: $gpu | free disk: $free GB"|Set-Content $Log
 $label.Text="Detected $ram GB RAM. Recommended: $recommended."+[Environment]::NewLine+"Downloading with up to 8 parallel connections..."
 [Windows.Forms.Application]::DoEvents()
 Download-Aria $url $model "Downloading $recommended" $min
 if($Vision){Download-Aria $projUrl $proj "Downloading image understanding support" 700MB}
 $label.Text="Setup complete. $recommended is installed locally.";$bar.Style="Continuous";$bar.Value=100;$cancel.Enabled=$false;"SUCCESS $(Get-Date -Format o)"|Add-Content $Log;[Windows.Forms.Application]::DoEvents();Start-Sleep -Seconds 2;exit 0
}catch{
 ("ERROR: "+$_.Exception.Message)|Add-Content $Log
 [Windows.Forms.MessageBox]::Show($_.Exception.Message+[Environment]::NewLine+[Environment]::NewLine+"Log: $Log","Doctor AI model setup",[Windows.Forms.MessageBoxButtons]::OK,[Windows.Forms.MessageBoxIcon]::Error)|Out-Null
 exit 10
}finally{if($form -and -not $form.IsDisposed){$form.Close();$form.Dispose()}}
