#define AppName "Doctor AI"
#define AppVersion "0.1.0"
#define AppPublisher "Doctor AI"
#define AppExeName "DoctorAI.exe"

[Setup]
AppId={{D8A4F5B2-8B1E-4B1B-8A32-DOCTORAI0001}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher={#AppPublisher}
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
Source: "Start-DoctorAI.ps1"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\Doctor AI"; Filename: "{app}\desktop\{#AppExeName}"
Name: "{autodesktop}\Doctor AI"; Filename: "{app}\desktop\{#AppExeName}"; Tasks: desktopicon

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Additional shortcuts:"

[Run]
Filename: "powershell.exe"; Parameters: "-NoProfile -ExecutionPolicy Bypass -File ""{app}\Start-DoctorAI.ps1"" -CheckOnly"; Flags: postinstall skipifsilent runhidden; Description: "Check Doctor AI prerequisites"
