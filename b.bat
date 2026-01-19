@echo off
setlocal
pushd "%~dp0\buddy" >nul
cargo build --target x86_64-pc-windows-gnu --release
set "ERR=%ERRORLEVEL%"
popd >nul
exit /b %ERR%
