build:
    cargo build --release

clippy *args:
    cargo clippy -- -D clippy::pedantic -D clippy::all {{ args }}

_fmt *args:
    cargo +nightly fmt {{ args }} -- --config imports_granularity="Crate"

fmt: _fmt

fmt-check: (_fmt '--check')
