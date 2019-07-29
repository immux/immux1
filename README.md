## Description

The master repository for Immux code, currently including:
- ImmuxDB, the immutable database, and
- Foldr, the immutable computing service.

## ImmuxDB

### Run the server

```
cargo build
cargo run
```

### Test


#### Unit tests
```
cargo test
```

To include print outs with `println!()` in test cases
```bash
cargo test -- --nocapture
```

#### End-to-end tests

```bash

# Remove existing data
cd /tmp
rm -rf immuxtest-*

# run an ImmuxDB instance in the background
./target/debug/immuxdb

# Execute test
# Note1. End-to-end tests are ignored by default
# Note2. Currently Immux does not support multi-threading, which means tests
# must be executed one by one.
cargo test -- --ignored --test-threads 1
```

#### Benchmarking

###### 1. Download

```bash
git submodule init
git submodule update
```

You only need to download data once.

###### 2. Execute benchmark

```bash
cargo bench
```

#### Profiling

##### 1. Install flamegraph

```
cargo install flamegraph
```

##### 2. Generate binary executable

```
cargo bench
```

###### 3. Run flamegraph

```bash
# Use sudo if prompted
flamegraph target/release/realistic-SOME_HASH
```

Inspect the "flamegraph.svg" in a web browser (it's interactive).

The exact executable name needs to be checked manually at the moment.
