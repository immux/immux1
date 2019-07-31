#![feature(test)]

extern crate test;
mod bench_core;

use bench_core::executor_benchmark::{benchmark_multi_insert, benchmark_single_insert};

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn bench_single_insert(b: &mut Bencher) {
        b.iter(|| benchmark_single_insert());
    }
}
