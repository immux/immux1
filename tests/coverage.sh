export CARGO_INCREMENTAL=0
export RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Cinline-threshold=0 -Clink-dead-code -Coverflow-checks=off -Zno-landing-pads"
cargo +nightly build --verbose $CARGO_OPTIONS
cargo +nightly test --lib --verbose $CARGO_OPTIONS

zip -0 ccov.zip `find . \( -name "libimmux*.gc*" \) -print`;
grcov ccov.zip -t lcov \
  --ignore-dir *github.com* \
  --ignore-dir *libcore* \
  --ignore-dir *rustc* \
  --ignore-dir *liballoc* \
  > lcov.info
genhtml -o tests/coverage-report/ --show-details --highlight --ignore-errors source --legend lcov.info

rm lcov.info
rm ccov.zip
