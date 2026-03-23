cargo build
mkdir dist 2> NUL
move ..\rTranslate_target\debug\rTranslate.exe .\dist\rTranslate_dbg.exe
move ..\rTranslate_target\debug\rTranslate.pdb .\dist\rTranslate_dbg.pdb
pause