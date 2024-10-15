call cfg.bat
cd ../
set RUSTFLAGS=--cfg=web_sys_unstable_apis
set RUSTFLAGS=-Zlocation-detail=none
set RUST_LOG=info
wasm-pack build --profiling  --target web --out-dir pkg_profiling1 --out-name wasm_engine
node build/build_wasm.js pkg_profiling1 wasm_engine
pause;
