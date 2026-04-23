/// Demonstrates MetricsCollector with 24 hourly slots simulating web-server traffic.
///
/// Each slot = one hour. Requests are recorded as they arrive; dashboards and
/// alerts query arbitrary hour ranges in O(log n) via segment trees.
use segment_tree::MetricsCollector;

fn main() {
    let mut collector = MetricsCollector::new(24);

    // (hour, request_count, base_response_ms, error_count)
    let hourly_data: &[(usize, u64, f64, u64)] = &[
        (0,   50,  80.0,  1),
        (1,   30,  75.0,  0),
        (2,   20,  70.0,  0),
        (3,   15,  65.0,  0),
        (4,   10,  60.0,  0),
        (5,   12,  62.0,  0),
        (6,   25,  70.0,  0),
        (7,   80,  95.0,  2),
        (8,  200, 120.0,  5),
        (9,  350, 150.0, 11),
        (10, 500, 200.0, 25), // morning peak
        (11, 480, 185.0, 20),
        (12, 400, 160.0, 12),
        (13, 350, 145.0,  8),
        (14, 300, 130.0,  6),
        (15, 320, 140.0,  7),
        (16, 380, 155.0,  9),
        (17, 450, 190.0, 18), // evening peak
        (18, 380, 170.0, 11),
        (19, 280, 140.0,  5),
        (20, 200, 110.0,  3),
        (21, 140,  95.0,  2),
        (22, 100,  90.0,  1),
        (23,  60,  85.0,  1),
    ];

    for &(hour, count, base_ms, errors) in hourly_data {
        for i in 0..count {
            let is_error = i < errors;
            // Small deterministic jitter so min ≠ max within a slot
            let response_ms = (base_ms + (i % 5) as f64 * 8.0 - 16.0).max(10.0);
            collector.record(hour, response_ms, is_error);
        }
    }

    println!("=== Web Server Metrics Dashboard (Segment Tree Demo) ===\n");

    let windows: &[(&str, usize, usize)] = &[
        ("Night         (00:00–05:59)", 0, 5),
        ("Morning       (06:00–11:59)", 6, 11),
        ("Afternoon     (12:00–17:59)", 12, 17),
        ("Evening       (18:00–23:59)", 18, 23),
        ("Business hrs  (09:00–17:59)", 9, 17),
        ("Full day      (00:00–23:59)", 0, 23),
    ];

    for &(label, from, to) in windows {
        let s = collector.query(from, to);
        print!("{label}  |  {:>5} req", s.total_requests);
        if s.total_requests > 0 {
            print!(
                "  |  errors {:>3} ({:4.1}%)  |  avg {:>6.1}ms  |  [{:.1} – {:.1}]ms",
                s.error_count,
                s.error_rate * 100.0,
                s.avg_response_ms.unwrap(),
                s.min_response_ms.unwrap(),
                s.max_response_ms.unwrap(),
            );
        }
        println!();
    }

    println!("\n--- Hourly breakdown ---");
    for hour in 0..24usize {
        let s = collector.query(hour, hour);
        if s.total_requests > 0 {
            println!(
                "{:02}:00  {:>4} req  avg {:>6.1}ms  [{:.1}–{:.1}ms]  err {}",
                hour,
                s.total_requests,
                s.avg_response_ms.unwrap(),
                s.min_response_ms.unwrap(),
                s.max_response_ms.unwrap(),
                s.error_count,
            );
        }
    }
}
