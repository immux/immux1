census90
========

`cenesus90` is the a subset of the Census data in 1990 of the United States of America.
See more information about the dataset at https://kdd.ics.uci.edu/databases/census1990/USCensus1990raw.html

The operations are inserting the data row-by-row, and then loading them back row-by-row with ID.

This benchmark primarily tests basic database operation for a medium size table (361 MB) of about 2.4 million rows.

Execution
---------

```bash
cargo bench --bench census90 -- 1000 100 1
```

`1000` is the row limit: Only the first 1000 rows are used;
`100` is the report period: performance statistics are reported for every 100 operations;
`1` is flag for correctness verification: if `0`, outputs are not verified; otherwise, outputs from the database
are compared against the original dataset.
