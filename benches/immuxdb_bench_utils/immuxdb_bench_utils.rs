use libimmuxdb::declarations::basics::Unit;
use std::error::Error;
use std::time::Instant;

pub type JsonTable = Vec<Unit>;

pub fn measure_iteration<D, F>(
    data: &[D],
    operate: F,
    operation_name: &str,
    report_period: usize,
) -> Result<Vec<(f64, f64)>, Box<dyn Error>>
where
    F: Fn(&D) -> Result<String, Box<dyn Error>>,
{
    let mut start = Instant::now();
    let mut count = 0;
    let total_periods = data.len() / report_period;
    let mut times: Vec<(f64, f64)> = Vec::with_capacity(total_periods + 1);
    for datum in data.iter() {
        operate(datum).unwrap();
        count += 1;
        if count % report_period == 0 {
            let elapsed = start.elapsed().as_millis();
            let average_time = elapsed as f64 / report_period as f64;
            println!(
                "took {}ms to execute {} {} operations, average {:.2}ms per item",
                elapsed, count, operation_name, average_time
            );
            start = Instant::now();
            times.push((count as f64, average_time));
        }
    }
    Ok(times)
}
