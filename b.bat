@echo off
setlocal
pushd "%~dp0\buddy" >nul
cargo build --release
set "ERR=%ERRORLEVEL%"
popd >nul
exit /b %ERR%
