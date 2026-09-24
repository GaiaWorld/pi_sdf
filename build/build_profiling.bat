call cfg.bat

cd ../
set RUSTFLAGS=--cfg=web_sys_unstable_apis
set RUST_LOG=info
wasm-pack build --profiling  --target web --out-dir pkg_profiling --out-name pi_sdf 
C:\\Users\\Administrator\\.cargo\\bin\\wasm-bindgen.exe ../../target/wasm32-unknown-unknown/release/pi_wasm_engine.wasm --out-dir pkg_profiling --typescript --target web --out-name pi_sdf
node build/build_wasm.js pkg_profiling pi_sdf
pause;


