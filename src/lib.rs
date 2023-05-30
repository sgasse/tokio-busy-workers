use std::{thread, time::Duration};

use prettytable::{row, Table};
use tokio::runtime::RuntimeMetrics;

pub fn monitor_worker_threads(metrics: RuntimeMetrics, update_interval: Duration) {
    let num_workers = metrics.num_workers();

    let mut worker_stats: Vec<_> = (0..num_workers)
        .map(|worker| WorkerStats::new(worker, &metrics))
        .collect();
    let mut deltas = worker_stats.clone();

    loop {
        thread::sleep(update_interval);

        println!("");
        for (stats, delta) in worker_stats.iter_mut().zip(deltas.iter_mut()) {
            let updated_stats = WorkerStats::new(stats.id, &metrics);
            *delta = updated_stats.delta(stats);
            *stats = updated_stats;
        }

        WorkerStats::print(deltas.iter());
    }
}

pub fn blocking_call_with_interruptions(duration: Duration) {
    let num_half_seconds = (duration.as_secs_f32() / 0.5).floor() as i32;

    for _ in 0..num_half_seconds {
        std::thread::sleep(Duration::from_millis(500));
    }
}

#[derive(Debug, Clone)]
struct WorkerStats {
    id: usize,
    busy_duration: Duration,
    poll_count: u64,
    worker_park_count: u64,
    local_schedule_count: u64,
    local_queue_depth: usize,
}

impl WorkerStats {
    fn new(id: usize, metrics: &RuntimeMetrics) -> Self {
        let busy_duration = metrics.worker_total_busy_duration(id);
        let poll_count = metrics.worker_poll_count(id);
        let worker_park_count = metrics.worker_park_count(id);
        let local_schedule_count = metrics.worker_local_schedule_count(id);
        let local_queue_depth = metrics.worker_local_queue_depth(id);

        Self {
            id,
            busy_duration,
            poll_count,
            worker_park_count,
            local_schedule_count,
            local_queue_depth,
        }
    }

    fn delta(&self, previous: &WorkerStats) -> WorkerStats {
        if self.id != previous.id {
            eprintln!(
                "Mismatch of thread IDs in delta: {} vs {}",
                self.id, previous.id
            );
        }

        WorkerStats {
            id: self.id,
            busy_duration: self.busy_duration - previous.busy_duration,
            poll_count: self.poll_count - previous.poll_count,
            worker_park_count: self.worker_park_count - previous.worker_park_count,
            local_schedule_count: self.local_schedule_count - previous.local_schedule_count,
            local_queue_depth: self.local_queue_depth - self.local_queue_depth,
        }
    }

    fn print<'a>(stats_iter: impl Iterator<Item = &'a WorkerStats>) {
        let mut table = Table::new();

        use prettytable::format;
        let format = format::FormatBuilder::new()
            .column_separator('|')
            .borders('|')
            .padding(1, 1)
            .build();
        table.set_format(format);

        table.add_row(row![
            "Delta for thread ID",
            "busy duration",
            "poll count",
            "worker park count",
            "local schedule count",
            "local queue depth",
        ]);

        for stats in stats_iter {
            table.add_row(row![
                r =>
                stats.id.to_string(),
                format!("{:?}", stats.busy_duration),
                format!("{:?}", stats.poll_count),
                format!("{:?}", stats.worker_park_count),
                format!("{:?}", stats.local_schedule_count),
                format!("{:?}", stats.local_queue_depth),
            ]);
        }

        table.printstd();
    }
}
