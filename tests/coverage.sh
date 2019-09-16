export CARGO_INCREMENTAL=0
export RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Cinline-threshold=0 -Clink-dead-code -Coverflow-checks=off -Zno-landing-pads"
cargo +nightly build --verbose $CARGO_OPTIONS
cargo +nightly test --lib --verbose $CARGO_OPTIONS

grcov . --ignore-dir deps -t lcov > lcov.info
genhtml -o tests/coverage-report/ --show-details --highlight --ignore-errors source --legend /tmp/lcov.info

rm lcov.info
