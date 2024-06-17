cd ../
cargo build --target=wasm32-unknown-unknown --release
"C:\\Users\\chuanyan\\AppData\\Local\\.wasm-pack\\wasm-bindgen-2b8061563077bfb8\\wasm-bindgen.exe" "D:\\work\\pi_show_wasm_bindgen\\pi_show_wasm_bindgen\\wasm_engine\\target\\wasm32-unknown-unknown\\release\\wasm_engine.wasm" "--out-dir" "D:\\work\\pi_show_wasm_bindgen\\pi_show_wasm_bindgen\\wasm_engine\\pkg_wasm_bindgen" "--typescript" "--target" "web" "--out-name" "wasm_engine"
node build/build_wasm.js pkg_wasm_bindgen wasm_engine
pause;