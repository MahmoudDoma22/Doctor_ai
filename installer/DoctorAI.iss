#define AppName "Doctor AI"
#define AppVersion "0.1.0"
#define AppPublisher "Doctor AI"
#define AppExeName "DoctorAI.exe"
#define PowerShellExe "{sys}\WindowsPowerShell\v1.0\powershell.exe"
#define LauncherArgs "-NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File ""{app}\Start-DoctorAI.ps1"""

[Setup]
AppId={{D8A4F5B2-8B1E-4B1B-8A32-0D0C0A100001}
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
Name: "{autoprograms}\Doctor AI"; Filename: "{#PowerShellExe}"; Parameters: "{#LauncherArgs}"; WorkingDir: "{app}"; IconFilename: "{app}\desktop\{#AppExeName}"
Name: "{autodesktop}\Doctor AI"; Filename: "{#PowerShellExe}"; Parameters: "{#LauncherArgs}"; WorkingDir: "{app}"; IconFilename: "{app}\desktop\{#AppExeName}"; Tasks: desktopicon

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Additional shortcuts:"

[Run]
Filename: "{#PowerShellExe}"; Parameters: "{#LauncherArgs}"; WorkingDir: "{app}"; Flags: postinstall skipifsilent; Description: "Start Doctor AI"
