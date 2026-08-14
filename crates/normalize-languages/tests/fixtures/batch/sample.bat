@echo off
REM A real-world-shaped batch script: subroutine labels, goto-based control
REM flow, and a real leaf label at end of file.

set NAME=world
echo Hello, %NAME%!

if exist output.txt goto :cleanup

goto :main

:main
echo running main
goto :cleanup

:cleanup
echo cleaning up
exit /b 0
