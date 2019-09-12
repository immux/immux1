berka99
========

`berka99` is operations data of Czech Bank

The operations are inserting the data row-by-row. In the future, more complex operations will be added.

See https://data.world/lpetrocelli/czech-financial-dataset-real-anonymized-transactions for more information about the data.

This benchmark aims to test database performance for financial records, cross-grouping data operations, and handling data updates.

Execution
---------

```bash
cargo bench --bench berka99 -- 1000 100 1
```

`1000` is the row limit: Only the first 1000 rows are used;
`100` is the report period: performance statistics are reported for every 100 operations;
`1` is flag for correctness verification: if `0`, outputs are not verified; otherwise, outputs from the database
are compared against the original dataset.
