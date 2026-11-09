# async-sqlite worker example

Shared Rust ledger queries running on native connection threads and in a dedicated
browser database worker, using the [async-sqlite fork](https://github.com/pgherveou/async-sqlite)
pinned in `Cargo.toml`. Browser storage uses OPFS.

Requires Rust, Node.js 20+, and wasm-bindgen CLI 0.2.129.

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129 --locked
npm ci
npx playwright install chromium
cargo test --locked
npm run test:browser
```

The tests cover transactions, rollback, value transport, caller responsiveness,
OPFS persistence and worker failures. Set `WASM_BINDGEN` or `CHROMIUM` to use
custom executables.
