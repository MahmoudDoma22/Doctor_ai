#define AppName "Doctor AI"
#define AppVersion "1.0.0"
#define AppExeName "DoctorAI.exe"
[Setup]
AppId={{7C6D5E4F-3A21-4B87-9D10-0A1B2C3D4E5F}
AppName={#AppName}
AppVersion={#AppVersion}
DefaultDirName={autopf}\Doctor AI
DefaultGroupName={#AppName}
OutputDir=..\installer-output
OutputBaseFilename=DoctorAI-Setup
Compression=lzma2
SolidCompression=yes
ArchitecturesInstallIn64BitMode=x64
PrivilegesRequired=admin
WizardStyle=modern
UninstallDisplayIcon={app}\desktop\{#AppExeName}
DisableProgramGroupPage=yes
[Files]
Source: "..\package\*"; DestDir: "{app}"; Flags: recursesubdirs createallsubdirs ignoreversion
Source: "Install-Model.ps1"; DestDir: "{app}"; Flags: ignoreversion
Source: "Start-DoctorAI.vbs"; DestDir: "{app}"; Flags: ignoreversion
[Icons]
Name: "{autoprograms}\Doctor AI"; Filename: "{app}\Start-DoctorAI.vbs"; WorkingDir: "{app}"; IconFilename: "{app}\desktop\{#AppExeName}"
Name: "{autodesktop}\Doctor AI"; Filename: "{app}\Start-DoctorAI.vbs"; WorkingDir: "{app}"; IconFilename: "{app}\desktop\{#AppExeName}"
[Run]
Filename: "{app}\prerequisites\vc_redist.x64.exe"; Parameters: "/install /quiet /norestart"; StatusMsg: "Installing Microsoft Visual C++ runtime..."; Flags: runhidden waituntilterminated
Filename: "{app}\Start-DoctorAI.vbs"; WorkingDir: "{app}"; Flags: postinstall skipifsilent; Description: "Launch Doctor AI"
[Code]
var RequirementsPage, ModulesPage, ModelPage: TWizardPage;
var RequirementsInfo, ModulesInfo, ModelInfo: TNewStaticText;
var ModelCheck, VisionCheck: TNewCheckBox;
procedure InitializeWizard;
begin
 RequirementsPage:=CreateCustomPage(wpWelcome,'Minimum system requirements','Review the minimum requirements before installing Doctor AI.');
 RequirementsInfo:=TNewStaticText.Create(RequirementsPage);RequirementsInfo.Parent:=RequirementsPage.Surface;RequirementsInfo.Left:=0;RequirementsInfo.Top:=ScaleY(8);RequirementsInfo.Width:=RequirementsPage.SurfaceWidth;RequirementsInfo.Height:=ScaleY(220);RequirementsInfo.AutoSize:=False;RequirementsInfo.WordWrap:=True;
 RequirementsInfo.Caption:='MINIMUM TO INSTALL THE DESKTOP'+#13#10+'• Windows 10 or 11, 64-bit'+#13#10+'• 4 GB RAM; modern 64-bit CPU (4 logical cores recommended)'+#13#10+'• 3 GB free disk space for app and bundled runtimes'+#13#10+#13#10+'MINIMUM FOR OFFLINE AI'+#13#10+'• 8 GB RAM for Gemma 4 E2B; 16 GB RAM recommended for Gemma 4 E4B'+#13#10+'• 7–10 GB free disk for model and optional image support'+#13#10+'• Internet for the first model download; after that, local chat works offline'+#13#10+#13#10+'GPU is optional. This release includes a CPU inference engine; GPU acceleration is not included yet.';
 ModulesPage:=CreateCustomPage(RequirementsPage.ID,'Offline modules and what they do','Review available local AI modules and hardware recommendations.');
 ModulesInfo:=TNewStaticText.Create(ModulesPage);ModulesInfo.Parent:=ModulesPage.Surface;ModulesInfo.Left:=0;ModulesInfo.Top:=ScaleY(8);ModulesInfo.Width:=ModulesPage.SurfaceWidth;ModulesInfo.Height:=ScaleY(300);ModulesInfo.AutoSize:=False;ModulesInfo.WordWrap:=True;
 ModulesInfo.Caption:='AVAILABLE IN THIS INSTALLER'+#13#10+'1. Offline Chat / Medical Assistant — Gemma 4 E2B or E4B. E2B suits 8–15 GB RAM; E4B suits 16 GB RAM and above.'+#13#10+#13#10+'2. Image Understanding — optional Gemma 4 vision projector for local image analysis; about 1 GB extra.'+#13#10+#13#10+'OPTIONAL CATALOG — NOT INSTALLED IN THIS RELEASE'+#13#10+'3. Whisper.cpp — offline speech-to-text; 8 GB RAM recommended.'+#13#10+'4. Piper — offline text-to-speech; lightweight CPU module, 4 GB RAM.'+#13#10+'5. Stable Diffusion / ComfyUI — image generation; 16 GB RAM + 8 GB dedicated VRAM recommended.'+#13#10+'6. Wan 2.x / ComfyUI — video generation; 32 GB RAM + 16–24 GB dedicated VRAM recommended.'+#13#10+'7. Offline audio generation — 32 GB RAM and a capable GPU recommended.'+#13#10+#13#10+'Modules marked catalog-only are not falsely presented as installed; their engines and workflows will be integrated separately.';
 ModelPage:=CreateCustomPage(ModulesPage.ID,'Recommended model and downloads','Doctor AI checks system memory and selects the best supported model.');
 ModelInfo:=TNewStaticText.Create(ModelPage);ModelInfo.Parent:=ModelPage.Surface;ModelInfo.Left:=0;ModelInfo.Top:=ScaleY(8);ModelInfo.Width:=ModelPage.SurfaceWidth;ModelInfo.Height:=ScaleY(116);ModelInfo.AutoSize:=False;ModelInfo.WordWrap:=True;
 ModelInfo.Caption:='AUTOMATIC MODEL RECOMMENDATION'+#13#10+'• 8–15 GB RAM: Gemma 4 E2B Q4_0 (lighter and faster).'+#13#10+'• 16 GB RAM or more: Gemma 4 E4B Q4_0 (stronger, higher memory use).'+#13#10+'• Below 8 GB RAM: install desktop only; offline AI is not recommended.'+#13#10+#13#10+'DOWNLOAD MANAGER: bundled aria2, up to 8 parallel connections, retries and resume support.';
 ModelCheck:=TNewCheckBox.Create(ModelPage);ModelCheck.Parent:=ModelPage.Surface;ModelCheck.Left:=0;ModelCheck.Top:=ScaleY(142);ModelCheck.Width:=ModelPage.SurfaceWidth;ModelCheck.Height:=ScaleY(30);ModelCheck.Caption:='Download and install the recommended offline chat model';ModelCheck.Checked:=True;
 VisionCheck:=TNewCheckBox.Create(ModelPage);VisionCheck.Parent:=ModelPage.Surface;VisionCheck.Left:=0;VisionCheck.Top:=ScaleY(178);VisionCheck.Width:=ModelPage.SurfaceWidth;VisionCheck.Height:=ScaleY(30);VisionCheck.Caption:='Also install image understanding support (about 1 GB extra)';VisionCheck.Checked:=True;
end;
procedure CurStepChanged(CurStep: TSetupStep);
var ResultCode: Integer; PowerShellPath,Params: String;
begin
 if (CurStep=ssPostInstall) and ModelCheck.Checked then begin
  PowerShellPath:=ExpandConstant('{sys}\WindowsPowerShell\v1.0\powershell.exe');
  Params:='-NoProfile -NonInteractive -ExecutionPolicy Bypass -STA -File "'+ExpandConstant('{app}\Install-Model.ps1')+'" -Aria2Path "'+ExpandConstant('{app}\download\aria2c.exe')+'"';
  if VisionCheck.Checked then Params:=Params+' -Vision';
  WizardForm.StatusLabel.Caption:='Checking hardware and preparing the recommended offline model...';
  if not Exec(PowerShellPath,Params,ExpandConstant('{app}'),SW_HIDE,ewWaitUntilTerminated,ResultCode) then MsgBox('Could not start the model downloader. The desktop is installed; run Install-Model.ps1 later.',mbError,MB_OK)
  else if ResultCode<>0 then MsgBox('The offline model was not installed (error '+IntToStr(ResultCode)+'). The desktop is still installed. Re-run the installer and retry the download.',mbInformation,MB_OK);
 end;
end;
