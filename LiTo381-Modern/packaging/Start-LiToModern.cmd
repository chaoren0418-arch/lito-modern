@echo off
rem LiTo Modern portable entry. Double-click this file after extracting the whole ZIP.
rem It needs only cmd.exe (no PowerShell) and starts LiToModern.exe from this folder.
setlocal
if not exist "%~dp0LiToModern.exe" goto missing
start "" /d "%~dp0." "%~dp0LiToModern.exe" %*
exit /b 0

:missing
echo LiToModern.exe was not found next to this file.
echo Extract the whole ZIP to a normal folder first, then run this file again.
pause
exit /b 1
