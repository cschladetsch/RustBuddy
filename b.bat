@echo off
setlocal EnableDelayedExpansion
pushd "%~dp0\buddy" >nul
set "CARGO_FEATURES="
if /i "%BUDDY_CUDA%"=="1" set "CARGO_FEATURES=--features cuda"
if defined CUDA_PATH set "CARGO_FEATURES=--features cuda"
if /i "%BUDDY_CUDA%"=="1" (
  if not defined BUDDY_CUDA_ARCH (
    for /f "usebackq tokens=1 delims=," %%a in (`nvidia-smi --query-gpu=compute_cap --format=csv 2^>NUL ^| findstr /r "^[0-9]"`) do (
      if not defined BUDDY_CUDA_ARCH set "BUDDY_CUDA_ARCH=%%a"
    )
  )
  if defined BUDDY_CUDA_ARCH (
    set "GGML_CUDA_ARCHITECTURES=!BUDDY_CUDA_ARCH!"
    set "GGML_CUDA_ARCHITECTURES=!GGML_CUDA_ARCHITECTURES: =!"
    set "GGML_CUDA_ARCHITECTURES=!GGML_CUDA_ARCHITECTURES:.=!"
    echo Using CUDA arch !GGML_CUDA_ARCHITECTURES! for whisper.cpp
  ) else (
    echo WARNING: Unable to detect CUDA compute capability. Set BUDDY_CUDA_ARCH, for example 75.
  )
)
cargo build --release %CARGO_FEATURES%
set "ERR=%ERRORLEVEL%"
popd >nul
exit /b %ERR%
