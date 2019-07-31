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
```
cargo test -- --nocapture
```

#### End-to-end tests

```
git submodule init
git submodule update
cd test

# Initiate a mongod instance / immuxdb instance yourself
mongod &

# to run all tests (takes a long time)
node test_mongo.js 

# to run some a subset of tests
node test_mongo.js core
```

### Profiling

#### Description

This is the manual instruction for profiling ImmuxDB in Ubuntu environment.

#### Prerequisite
- Make sure you have [VirtualBox](https://www.virtualbox.org/) installed.
- Launch [Ubuntu](http://releases.ubuntu.com/18.04/) (ubuntu-18.04.2-amd64) from VirtualBox.
- Install git and pull the current project, and checkout `benchmarks` branch.

#### Setup
- Run `setup.sh`
- Add this piece of code to `cargo.toml`

```
[profile.bench]
debug = true
```

#### Profiling bench function

- `rustup run nightly cargo bench`, it might take a while for finish running, once it is done, check the new gnerated binary, For example:

```
Running target/release/deps/lib-8403c8edede1de55
```

- `valgrind --tool=callgrind --dump-instr=yes --collect-jumps=yes --simulate-cache=yes target/release/deps/{#HASH_NUMBER} {#BENCH_FUNCTION_NAME}`

replase the `{#HASH_NUMBER}` and `{#BENCH_FUNCTION_NAME}`, for example:

```
- `valgrind --tool=callgrind --dump-instr=yes --collect-jumps=yes --simulate-cache=yes Running target/release/deps/lib-8403c8edede1de55 bench_add_two`
```

- Eventually it will generate a `callgrind.out.<pid>`, use kcachegrind open `callgrind.out.<pid>`

- You can modify the bench function in 
