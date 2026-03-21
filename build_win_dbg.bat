cargo build
mkdir dist 2> NUL
move ..\rTranslate_target\debug\rTranslate.exe .\dist\rTranslate_dbg.exe
pause