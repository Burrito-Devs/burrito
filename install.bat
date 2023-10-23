@ECHO OFF
set burrito_dir=%HOMEPATH%\.burrito
mkdir %burrito_dir%
mkdir %burrito_dir%\sounds
xcopy data\sounds %burrito_dir%\sounds /E /I /Y 
copy /Y data\systems.json %burrito_dir%
copy /Y burrito.exe %HOMEPATH%
