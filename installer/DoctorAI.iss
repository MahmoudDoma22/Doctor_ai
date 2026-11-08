#define AppName "Doctor AI"
#define AppVersion "2.0.0"
#define AppExeName "DoctorAI.exe"
[Setup]
AppId={{6A2D7D41-9F51-4C86-AC21-2C0B65A8F201}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher=Doctor AI
DefaultDirName={autopf}\Doctor AI
DefaultGroupName={#AppName}
OutputDir=..\installer-output
OutputBaseFilename=DoctorAI-Setup
SetupIconFile=..\package\DoctorAI.ico
UninstallDisplayIcon={app}\{#AppExeName}
ArchitecturesInstallIn64BitMode=x64
PrivilegesRequired=admin
WizardStyle=modern
Compression=lzma2
SolidCompression=yes
DisableProgramGroupPage=yes
[Files]
Source: "..\package\DoctorAI.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\package\runtime\mistralrs\mistralrs.exe"; DestDir: "{app}\runtime\mistralrs"; Flags: ignoreversion
[Icons]
Name: "{autoprograms}\Doctor AI"; Filename: "{app}\{#AppExeName}"; WorkingDir: "{app}"
Name: "{autodesktop}\Doctor AI"; Filename: "{app}\{#AppExeName}"; WorkingDir: "{app}"; Tasks: desktopicon
[Tasks]
Name: "desktopicon"; Description: "إنشاء اختصار على سطح المكتب"; GroupDescription: "اختصارات إضافية:"
[Run]
Filename: "{app}\{#AppExeName}"; Description: "تشغيل Doctor AI"; Flags: postinstall nowait skipifsilent
[Messages]
WelcomeLabel2=سيتم تثبيت Doctor AI بواجهة Rust أصلية. يمكنك تغيير مجلد التثبيت في الخطوة التالية.
SelectDirDesc=اختر مكان تثبيت البرنامج. لا يتم استخدام PowerShell أثناء التثبيت.
