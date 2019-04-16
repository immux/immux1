# Description

Unumdb is a database which works like git.

# Run the server

```
cargo build
cargo run
```

# Test


## For native testing
```
cargo test
```

To include print outs with `println!()` in test cases
```
cargo test -- --nocapture
```

## For testing our own implementation against official server

```
git submodule init
git submodule update
cd test

# Initiate a mongod instance / unumdb instance yourself
mongod &

# to run all tests (takes a long time)
node test_mongo.js 

# to run some a subset of tests
node test_mongo.js core
```

Example of outputs (tested against official server):

```
 $ node test_mongo.js core                                                                                                                                             [21:38:10]
Running 1026 tests from /Users/andy/code/unumdb/test/mongo/jstests/core
Fail [0|1|1026] - /Users/andy/code/unumdb/test/mongo/jstests/core/agg_hint.js - error Error: Command failed: mongo /Users/andy/code/unumdb/test/mongo/jstests/core/agg_hint.js

OK [1|2|1026] - /Users/andy/code/unumdb/test/mongo/jstests/core/apitest_db_profile_level.js
Fail [1|3|1026] - /Users/andy/code/unumdb/test/mongo/jstests/core/apply_ops_invalid_index_spec.js - error Error: Command failed: mongo /Users/andy/code/unumdb/test/mongo/jstests/core/apply_ops_invalid_index_spec.js

OK [2|4|1026] - /Users/andy/code/unumdb/test/mongo/jstests/core/aggregation_getmore_batchsize.js
OK [3|5|1026] - /Users/andy/code/unumdb/test/mongo/jstests/core/apply_ops_without_ns.js
OK [4|6|1026] - /Users/andy/code/unumdb/test/mongo/jstests/core/apply_ops_index_collation.js
OK [5|7|1026] - /Users/andy/code/unumdb/test/mongo/jstests/core/apply_ops_dups.js
OK [6|8|1026] - /Users/andy/code/unumdb/test/mongo/jstests/core/all3.js
OK [7|9|1026] - /Users/andy/code/unumdb/test/mongo/jstests/core/and2.js
OK [8|10|1026] - /Users/andy/code/unumdb/test/mongo/jstests/core/apply_ops2.js
OK [9|11|1026] - /Users/andy/code/unumdb/test/mongo/jstests/core/apitest_db.js
OK [10|12|1026] - /Users/andy/code/unumdb/test/mongo/jstests/core/all4.js
OK [11|13|1026] - /Users/andy/code/unumdb/test/mongo/jstests/core/array_match3.js
OK [12|14|1026] - /Users/andy/code/unumdb/test/mongo/jstests/core/array3.js
OK [13|15|1026] - /Users/andy/code/unumdb/test/mongo/jstests/core/array_match2.js
OK [14|16|1026] - /Users/andy/code/unumdb/test/mongo/jstests/core/array_match1.js
OK [15|17|1026] - /Users/andy/code/unumdb/test/mongo/jstests/core/array1.js
OK [16|18|1026] - /Users/andy/code/unumdb/test/mongo/jstests/core/array_match4.js
OK [17|19|1026] - /Users/andy/code/unumdb/test/mongo/jstests/core/all5.js
OK [18|20|1026] - /Users/andy/code/unumdb/test/mongo/jstests/core/and3.js
OK [19|21|1026] - /Users/andy/code/unumdb/test/mongo/jstests/core/arrayfind3.js
OK [20|22|1026] - /Users/andy/code/unumdb/test/mongo/jstests/core/array4.js
OK [21|23|1026] - /Users/andy/code/unumdb/test/mongo/jstests/core/arrayfind2.js
OK [22|24|1026] - /Users/andy/code/unumdb/test/mongo/jstests/core/all.js
Fail [22|25|1026] - /Users/andy/code/unumdb/test/mongo/jstests/core/arrayfind8.js - error Error: Command failed: mongo /Users/andy/code/unumdb/test/mongo/jstests/core/arrayfind8.js

OK [23|26|1026] - /Users/andy/code/unumdb/test/mongo/jstests/core/arrayfind1.js
OK [24|27|1026] - /Users/andy/code/unumdb/test/mongo/jstests/core/arrayfind4.js
OK [25|28|1026] - /Users/andy/code/unumdb/test/mongo/jstests/core/auth2.js
OK [26|29|1026] - /Users/andy/code/unumdb/test/mongo/jstests/core/arrayfind6.js
OK [27|30|1026] - /Users/andy/code/unumdb/test/mongo/jstests/core/automation_setparameter.js
OK [28|31|1026] - /Users/andy/code/unumdb/test/mongo/jstests/core/autocomplete.js
OK [29|32|1026] - /Users/andy/code/unumdb/test/mongo/jstests/core/arrayfinda.js
OK [30|33|1026] - /Users/andy/code/unumdb/test/mongo/jstests/core/bad_index_plugin.js
```

# For tracking package between official mongo server and client

The branch is (here)[https://github.com/immux/unumdb/blob/mitm-proxy/tools/mitm.js]

To run the proxy server:

```
mongod
```

```
node mitm.js
```

```
mongo --port 26000
```
