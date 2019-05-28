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
