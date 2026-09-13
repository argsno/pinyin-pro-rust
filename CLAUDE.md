# Repository Instructions

## Verify

After changing anything, run:

```bash
cargo test
cargo clippy --all-targets
```

Keep zero warnings. `napi`/`wasm` feature code must stay
compiling: `cargo check --features napi,wasm`.
