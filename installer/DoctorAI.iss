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

[Code]
var
  RequirementsPage, ModulesPage: TWizardPage;
  RequirementsText, ModulesText: TNewStaticText;

procedure InitializeWizard;
begin
  RequirementsPage := CreateCustomPage(wpWelcome, 'متطلبات التشغيل', 'راجع متطلبات الجهاز قبل اختيار مكان التثبيت.');
  RequirementsText := TNewStaticText.Create(RequirementsPage);
  RequirementsText.Parent := RequirementsPage.Surface;
  RequirementsText.Left := 0;
  RequirementsText.Top := ScaleY(8);
  RequirementsText.Width := RequirementsPage.SurfaceWidth;
  RequirementsText.Height := ScaleY(190);
  RequirementsText.AutoSize := False;
  RequirementsText.WordWrap := True;
  RequirementsText.Caption :=
    'التطبيق: Windows 10 أو 11 بنظام 64-bit.' + #13#10 +
    'الحد الأدنى المقترح للمحادثة المحلية: 8 GB RAM مع Gemma 4 E2B.' + #13#10 +
    'الموديل الأكبر Gemma 4 E4B يحتاج ذاكرة أكبر؛ 16 GB RAM أو أكثر موصى بها.' + #13#10 +
    'اترك مساحة إضافية على SSD لتنزيل ملفات الموديلات (عدة جيجابايت لكل موديل).' + #13#10 +
    'يتم فحص مواصفات الجهاز مرة أخرى داخل التطبيق قبل اختيار الموديل.';

  ModulesPage := CreateCustomPage(RequirementsPage.ID, 'الموديولات والبرامج الاختيارية', 'تعرف على الموديولات قبل إكمال التثبيت.');
  ModulesText := TNewStaticText.Create(ModulesPage);
  ModulesText.Parent := ModulesPage.Surface;
  ModulesText.Left := 0;
  ModulesText.Top := ScaleY(8);
  ModulesText.Width := ModulesPage.SurfaceWidth;
  ModulesText.Height := ScaleY(250);
  ModulesText.AutoSize := False;
  ModulesText.WordWrap := True;
  ModulesText.Caption :=
    'المحادثة المحلية: محرك Rust مرفق؛ ملفات الموديل تختارها وتحملها من داخل التطبيق.' + #13#10 + #13#10 +
    'Whisper.cpp: تحويل الكلام إلى نص محلياً.' + #13#10 +
    'Piper: تحويل النص إلى صوت محلياً.' + #13#10 +
    'ComfyUI: تشغيل Workflows وتوليد الصور.' + #13#10 +
    'Wan 2.x: توليد الفيديو؛ يحتاج GPU وذاكرة فيديو كبيرة.' + #13#10 + #13#10 +
    'Krita وBlender وShotcut وKdenlive برامج مجانية اختيارية وروابطها داخل التطبيق.' + #13#10 +
    'الموديولات الإضافية لا تُعرض على أنها مثبتة إلا بعد إعدادها فعلياً.';
end;
