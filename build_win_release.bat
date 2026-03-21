cargo build --target i686-pc-windows-msvc --release
mkdir dist 2> NUL
move ..\rTranslate_target\i686-pc-windows-msvc\release\rTranslate.exe .\dist\rTranslate.exe
REM move .\target\release\rTranslate.exe .\dist\rTranslate.exe
pause