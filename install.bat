@ECHO OFF
set burrito_dir=%HOMEPATH%\.burrito
xcopy data "%burrito_dir%" /E /I /Y 
xcopy burrito.exe "%HOMEPATH%" /Y
