@ECHO OFF
set burrito_dir=%HOMEPATH%\.burrito
mkdir burrito_dir
xcopy data\sounds %burrito_dir%
copy data\systems.json %burrito_dir%
copy burrito.exe %HOMEPATH%
