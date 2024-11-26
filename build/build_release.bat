@REM set RUSTFLAGS=--cfg=web_sys_unstable_apis
@REM set RUSTFLAGS=-Zlocation-detail=none
set RUST_LOG=info
call cfg.bat
cd ../
wasm-pack build --release  --target web --out-dir pkg --out-name pi_sdf
node build/build_wasm.js pkg pi_sdf
pause;