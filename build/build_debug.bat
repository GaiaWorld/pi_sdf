call cfg.bat

cd ../
set RUSTFLAGS=--cfg=web_sys_unstable_apis
set RUST_LOG=info
wasm-pack build --dev  --target web --out-dir pkg_debug --out-name pi_sdf
node build/build_wasm.js pkg_debug pi_sdf
pause;

