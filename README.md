# nightscout-rs

Alpha Rust client and typed models for the Nightscout v3 API.

It currently handles:

- Nightscout bearer-token authentication from a permission role
- posting v3 documents with one bearer refresh retry on 401
- entry, treatment, devicestatus, profile, and related model types
- small dedup lookup helpers beyond the default Nightscout Model

## Development

```bash
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```
