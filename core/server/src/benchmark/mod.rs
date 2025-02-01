use csv::Writer;
use std::fs::OpenOptions;

pub fn log_benchmark(endpoint: &str, latency: u128, middleware_type: &str) {
    let mut wtr = Writer::from_writer(
        OpenOptions::new()
            .append(true)
            .create(true)
            .open("src/benchmark/benchmark_results.csv")
            .unwrap(),
    );

    wtr.write_record(&[endpoint, &latency.to_string()]).unwrap();
    wtr.flush().unwrap();
}
