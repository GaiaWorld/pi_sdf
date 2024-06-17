call cfg.bat

cd ../
set RUSTFLAGS=--cfg=web_sys_unstable_apis
set RUST_LOG=info
wasm-pack build --profiling  --target web --out-dir pkg_profiling --out-name wasm_engine --features release
C:\\Users\\chuanyan\\AppData\\Local\\.wasm-pack\\wasm-bindgen-df7a1317af78a1c0\\wasm-bindgen.exe ../../target/wasm32-unknown-unknown/release/pi_wasm_engine.wasm --out-dir pkg_profiling --typescript --target web --out-name wasm_engine
node build/build_wasm.js pkg_profiling wasm_engine
pause;
