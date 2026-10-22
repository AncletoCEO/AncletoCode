#![cfg(unix)]
#![allow(clippy::unwrap_used)]

use std::time::{Duration, Instant};

use ancleto_code_cmd::vt::TerminalState;

const ENGINE_START_P99_MS: f64 = 50.0;
const FEED_BYTES: usize = 5 * 1024 * 1024;
const FEED_BUDGET: Duration = Duration::from_secs(5);
const SNAPSHOT_P99_MS: f64 = 50.0;
const SEARCH_BUDGET: Duration = Duration::from_secs(1);
const INPUT_TO_BUFFER_P99_MS: f64 = 20.0;

fn percentile(samples: &mut [f64], p: f64) -> f64 {
    samples.sort_by(|left, right| left.total_cmp(right));
    if samples.is_empty() {
        return 0.0;
    }
    let index = ((samples.len() - 1) as f64 * p).round() as usize;
    samples[index.min(samples.len() - 1)]
}

#[test]
fn vt_engine_meets_headless_performance_thresholds() {
    let mut startup = Vec::new();
    for _ in 0..30 {
        let started = Instant::now();
        let terminal = TerminalState::new(120, 40);
        std::hint::black_box(terminal);
        startup.push(started.elapsed().as_secs_f64() * 1000.0);
    }
    let start_p99 = percentile(&mut startup, 0.99);
    assert!(
        start_p99 < ENGINE_START_P99_MS,
        "startup p99 {start_p99:.2} ms >= {ENGINE_START_P99_MS} ms"
    );

    let mut terminal = TerminalState::new(120, 40);
    let line = "x".repeat(120);
    let chunk = format!("{line}\r\n").repeat(16);
    let started = Instant::now();
    let mut fed = 0;
    while fed < FEED_BYTES {
        terminal.feed(chunk.as_bytes());
        fed += chunk.len();
    }
    let feed_time = started.elapsed();
    assert!(
        feed_time < FEED_BUDGET,
        "feed {FEED_BYTES} bytes took {feed_time:?} >= {FEED_BUDGET:?}"
    );

    let mut snapshots = Vec::new();
    for _ in 0..100 {
        let started = Instant::now();
        std::hint::black_box(terminal.snapshot());
        snapshots.push(started.elapsed().as_secs_f64() * 1000.0);
    }
    let snapshot_p99 = percentile(&mut snapshots, 0.99);
    assert!(
        snapshot_p99 < SNAPSHOT_P99_MS,
        "snapshot p99 {snapshot_p99:.2} ms >= {SNAPSHOT_P99_MS} ms"
    );

    let search_started = Instant::now();
    let matches = terminal.search("xxxxx", false);
    let search_time = search_started.elapsed();
    assert!(
        search_time < SEARCH_BUDGET,
        "search took {search_time:?} >= {SEARCH_BUDGET:?}"
    );
    assert!(!matches.is_empty());

    let mut input = Vec::new();
    for _ in 0..200 {
        let started = Instant::now();
        terminal.feed(b"a");
        std::hint::black_box(terminal.snapshot());
        input.push(started.elapsed().as_secs_f64() * 1000.0);
    }
    let input_p99 = percentile(&mut input, 0.99);
    assert!(
        input_p99 < INPUT_TO_BUFFER_P99_MS,
        "input-to-buffer p99 {input_p99:.2} ms >= {INPUT_TO_BUFFER_P99_MS} ms"
    );
}
