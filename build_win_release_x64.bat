cargo build --release
mkdir dist 2> NUL
move ..\rTranslate_target\release\rTranslate.exe .\dist\rTranslate_x64.exe
REM move .\target\release\rTranslate.exe .\dist\rTranslate.exe
pause