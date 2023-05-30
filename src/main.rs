use std::time::Duration;

use anyhow::Result;
use tokio::{runtime, task::JoinSet};
use tokio_busy_workers::{blocking_call_with_interruptions, monitor_worker_threads};

fn main() -> Result<()> {
    let rt = runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_time()
        .build()?;

    let metrics = rt.metrics();

    std::thread::spawn(move || {
        monitor_worker_threads(metrics, Duration::from_secs(5));
    });

    rt.block_on(run_idle_worker());
    // rt.block_on(run_all_workers_busy_no_blocking());
    // rt.block_on(run_all_workers_busy_blocking1());
    // rt.block_on(run_all_workers_busy_blocking2());

    Ok(())
}

async fn run_idle_worker() {
    let mut set = JoinSet::new();

    // We have less tasks than workers, leading to idle workers.
    for _ in 0..3 {
        set.spawn(async {
            loop {
                let _now = std::time::Instant::now();
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        });
    }

    set.join_next().await;
}

async fn run_all_workers_busy_no_blocking() {
    let mut set = JoinSet::new();

    // We have far more tasks than workers, all workers are busy.
    for _ in 0..100 {
        set.spawn(async {
            loop {
                let _now = std::time::Instant::now();
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        });
    }

    set.join_next().await;
}

async fn run_all_workers_busy_blocking1() {
    let mut set = JoinSet::new();

    // We have far more tasks than workers, all workers are busy.
    for _ in 0..100 {
        set.spawn(async {
            loop {
                let _now = std::time::Instant::now();
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        });
    }

    // This tasks does not yield for 20s at a time with one sleep call.
    set.spawn(async {
        loop {
            std::thread::sleep(Duration::from_secs(20));
        }
    });

    set.join_next().await;
}

async fn run_all_workers_busy_blocking2() {
    let mut set = JoinSet::new();

    // We have far more tasks than workers, all workers are busy.
    for _ in 0..100 {
        set.spawn(async {
            loop {
                let _now = std::time::Instant::now();
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        });
    }

    // This tasks does not yield for 20s at a time with several sleep calls.
    set.spawn(async {
        loop {
            blocking_call_with_interruptions(Duration::from_secs(20));
        }
    });

    set.join_next().await;
}
