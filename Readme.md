# Scribe

[![Rust](https://github.com/bartossh/scribe/actions/workflows/rust.yml/badge.svg)](https://github.com/bartossh/scribe/actions/workflows/rust.yml)

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

### Integration tests

Remember to run the server with default settings:

```sh
cargo run --release
```

```sh
cargo test --profile test --test integration_tests -v -- --nocapture --ignored --test-threads=1
```

### Run dependencies

Project uses MongoDB as a data repository.

The `your_user_name` default is `scribe`.
The `your_password` default is `scribe`.

1. To run dockerized database for development:

```sh
MONGO_DB_USER=your_user_name MONGO_DB_PASSWORD=your_password docker compose -f dependencies/docker-compose.development.yaml up -d
```

You can vew database in the browser app at `http://localhost:8081/`.


1. To run dockerized database for staging or/and prod:

```sh
MONGO_DB_USER=your_user_name MONGO_DB_PASSWORD=your_password docker compose -f dependencies/docker-compose.yaml up -d
```

Database is then not exposed to the outside word via browser app at `localhost`.

### Run server

1. Default on addr: `0.0.0.0:8000`

```sh
cargo run --release 
```

2. With `setup.yaml` file path as the argument. The file contains the setup parameters. Look in to `default.yaml` for a reference.

```sh
cargo run --release setup.yaml
```

