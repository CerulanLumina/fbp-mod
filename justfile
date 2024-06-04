fmt:
    cargo +nightly fmt -- --config imports_granularity="Crate"
clippy *args:
    cargo +nightly clippy {{args}} -- -W clippy::pedantic -W clippy::all
