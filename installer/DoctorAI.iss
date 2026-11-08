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
[Files]
Source: "..\package\*"; DestDir: "{app}"; Flags: recursesubdirs createallsubdirs ignoreversion
[Icons]
Name: "{autoprograms}\Doctor AI"; Filename: "{sys}\WindowsPowerShell\v1.0\powershell.exe"; Parameters: "-NoProfile -ExecutionPolicy Bypass -File ""{app}\Start-DoctorAI.ps1"""; WorkingDir: "{app}"; IconFilename: "{app}\desktop\{#AppExeName}"
Name: "{autodesktop}\Doctor AI"; Filename: "{sys}\WindowsPowerShell\v1.0\powershell.exe"; Parameters: "-NoProfile -ExecutionPolicy Bypass -File ""{app}\Start-DoctorAI.ps1"""; WorkingDir: "{app}"; IconFilename: "{app}\desktop\{#AppExeName}"
[Run]
Filename: "{sys}\WindowsPowerShell\v1.0\powershell.exe"; Parameters: "-NoProfile -ExecutionPolicy Bypass -File ""{app}\Start-DoctorAI.ps1"""; WorkingDir: "{app}"; Flags: postinstall skipifsilent; Description: "Launch Doctor AI"
