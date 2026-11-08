Option Explicit
Dim shell, fso, root, launcher, command
Set shell = CreateObject("WScript.Shell")
Set fso = CreateObject("Scripting.FileSystemObject")
root = fso.GetParentFolderName(WScript.ScriptFullName)
launcher = root & "\Start-DoctorAI.ps1"

If Not fso.FileExists(launcher) Then
  MsgBox "Doctor AI startup component is missing.", vbCritical, "Doctor AI"
  WScript.Quit 2
End If

command = "powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File """ & launcher & """"
shell.Run command, 0, False
