# Scribe

Yet another logs server, that keeps logs in binary-packed format.

## Why

1. To store logs packed in binary format for fast writes.
2. To save disc space.
3. To save on indexing large dictionaries.

## Development

Developed with `rustc 1.77.2` and `cargo 1.77.2`.

### Test

```sh
cargo test --profile test -- --nocapture --test-threads=1
```
