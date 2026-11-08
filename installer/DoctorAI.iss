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
Source: "Install-Gemma4.ps1"; DestDir: "{app}"; Flags: ignoreversion
Source: "Start-DoctorAI.vbs"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\Doctor AI"; Filename: "{app}\Start-DoctorAI.vbs"; WorkingDir: "{app}"; IconFilename: "{app}\desktop\{#AppExeName}"
Name: "{autodesktop}\Doctor AI"; Filename: "{app}\Start-DoctorAI.vbs"; WorkingDir: "{app}"; IconFilename: "{app}\desktop\{#AppExeName}"

[Run]
Filename: "{app}\Start-DoctorAI.vbs"; WorkingDir: "{app}"; Flags: postinstall skipifsilent; Description: "Launch Doctor AI"

[Code]
var
  ModelPage: TWizardPage;
  ModelCheck: TNewCheckBox;
  ModelInfo: TNewStaticText;

procedure InitializeWizard;
begin
  ModelPage := CreateCustomPage(wpSelectDir,
    'AI model setup',
    'Choose whether to install the recommended local AI model now.');

  ModelInfo := TNewStaticText.Create(ModelPage);
  ModelInfo.Parent := ModelPage.Surface;
  ModelInfo.Left := 0;
  ModelInfo.Top := ScaleY(8);
  ModelInfo.Width := ModelPage.SurfaceWidth;
  ModelInfo.Height := ScaleY(92);
  ModelInfo.AutoSize := False;
  ModelInfo.WordWrap := True;
  ModelInfo.Caption :=
    'Doctor AI can use Ollama to run Gemma 4 E4B locally.' + #13#10 +
    'The model download is several gigabytes and requires an internet connection and enough free disk space.' + #13#10 +
    'Ollama must be installed. If it is missing, the installer will explain how to get it.';

  ModelCheck := TNewCheckBox.Create(ModelPage);
  ModelCheck.Parent := ModelPage.Surface;
  ModelCheck.Left := 0;
  ModelCheck.Top := ScaleY(116);
  ModelCheck.Width := ModelPage.SurfaceWidth;
  ModelCheck.Height := ScaleY(36);
  ModelCheck.Caption := 'Download and install Gemma 4 E4B after copying Doctor AI';
  ModelCheck.Checked := True;
end;

function ShouldInstallGemma: Boolean;
begin
  Result := ModelCheck.Checked;
end;

procedure CurStepChanged(CurStep: TSetupStep);
var
  ResultCode: Integer;
  PowerShellPath, Params: String;
begin
  if (CurStep = ssPostInstall) and ModelCheck.Checked then
  begin
    PowerShellPath := ExpandConstant('{sys}\WindowsPowerShell\v1.0\powershell.exe');
    Params := '-NoProfile -NonInteractive -ExecutionPolicy Bypass -File "' +
      ExpandConstant('{app}\Install-Gemma4.ps1') + '"';
    WizardForm.StatusLabel.Caption := 'Downloading Gemma 4 E4B. This may take several minutes...';
    if not Exec(PowerShellPath, Params, ExpandConstant('{app}'),
      SW_HIDE, ewWaitUntilTerminated, ResultCode) then
    begin
      MsgBox('Could not start the Gemma 4 setup step. You can run Install-Gemma4.ps1 later from the Doctor AI folder.',
        mbError, MB_OK);
    end
    else if ResultCode <> 0 then
    begin
      MsgBox('Gemma 4 could not be installed (error ' + IntToStr(ResultCode) + ').' + #13#10 +
        'Doctor AI is installed. Install or start Ollama, then run Install-Gemma4.ps1 from the Doctor AI installation folder.' + #13#10 +
        'Ollama download: https://ollama.com/download',
        mbError, MB_OK);
    end;
  end;
end;
