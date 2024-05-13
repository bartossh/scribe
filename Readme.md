# Scribe

Yet another logs server, that keeps logs in binary-packed format.

## Why

1. To store logs packed in binary format for fast writes.
2. To save disc space.
3. To save on indexing large dictionaries.

## Development

Developed with `rustc 1.77.2` and `cargo 1.77.2`.

### Unit Tests 

```sh
cargo test --profile test -- --nocapture --test-threads=1
```

### Run server

1. Default on addr: `0.0.0.0:8000`

```sh
cargo run --release 
```

2. With `setup.yaml` file path as the argument. The file contains the setup parameters. Look in to `default.yaml` for a reference.

```sh
cargo run --release setup.yaml
```
