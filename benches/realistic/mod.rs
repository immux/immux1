mod berka99;
mod census90;
mod realistic_bench;
mod utils;

pub use berka99::berka99;
pub use census90::census90;

pub use realistic_bench::{bench_all, BenchSpec};

fn main() {
    let benches: Vec<BenchSpec> = vec![
        BenchSpec {
            name: "census90",
            unicus_port: 10001,
            main: &census90,
            row_limit: 20_000,
            report_period: 1_000,
        },
        BenchSpec {
            name: "berka99",
            unicus_port: 10002,
            main: &berka99,
            row_limit: 5_000,
            report_period: 1_000,
        },
    ];
    bench_all(&benches);
}
