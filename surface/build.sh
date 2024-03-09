RUSTFLAGS=--cfg=web_sys_unstable_apis cargo build --no-default-features --target wasm32-unknown-unknown \
--bin surface

# Generate bindings
for i in target/wasm32-unknown-unknown/debug/*.wasm;
do
    wasm-bindgen --no-typescript --out-dir html --web "$i";
done
