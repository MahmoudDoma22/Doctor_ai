Option Explicit
Dim shell, fso, root, javaPath, jarPath, appPath, command, result
Set shell = CreateObject("WScript.Shell")
Set fso = CreateObject("Scripting.FileSystemObject")
root = fso.GetParentFolderName(WScript.ScriptFullName)
javaPath = root & "\runtime\bin\java.exe"
jarPath = root & "\backend\doctor-ai-backend.jar"
appPath = root & "\desktop\DoctorAI.exe"

If Not fso.FileExists(javaPath) Then
  MsgBox "Doctor AI Java runtime is missing.", vbCritical, "Doctor AI"
  WScript.Quit 2
End If
If Not fso.FileExists(jarPath) Then
  MsgBox "Doctor AI backend is missing.", vbCritical, "Doctor AI"
  WScript.Quit 3
End If
If Not fso.FileExists(appPath) Then
  MsgBox "Doctor AI desktop application is missing.", vbCritical, "Doctor AI"
  WScript.Quit 4
End If

On Error Resume Next
result = shell.Run("powershell.exe -NoProfile -NonInteractive -Command ""try { Invoke-RestMethod 'http://127.0.0.1:8787/api/health' -TimeoutSec 2 | Out-Null; exit 0 } catch { exit 1 }""", 0, True)
On Error GoTo 0
If result <> 0 Then
  command = """" & javaPath & """ -jar """ & jarPath & """"
  shell.Run command, 0, False
End If
shell.Run """" & appPath & """", 1, False
