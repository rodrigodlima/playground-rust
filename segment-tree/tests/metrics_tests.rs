use segment_tree::MetricsCollector;

#[test]
fn basic_recording_and_query() {
    let mut mc = MetricsCollector::new(10);
    mc.record(0, 100.0, false);
    mc.record(0, 200.0, false);
    mc.record(1, 150.0, true);

    let s = mc.query(0, 1);
    assert_eq!(s.total_requests, 3);
    assert_eq!(s.error_count, 1);
    assert!((s.error_rate - 1.0 / 3.0).abs() < 1e-9);
    assert_eq!(s.min_response_ms, Some(100.0));
    assert_eq!(s.max_response_ms, Some(200.0));
    // avg = (100 + 200 + 150) / 3 = 150
    assert!((s.avg_response_ms.unwrap() - 150.0).abs() < 1e-9);
}

#[test]
fn empty_range_returns_none() {
    let mc = MetricsCollector::new(10);
    let s = mc.query(0, 9);
    assert_eq!(s.total_requests, 0);
    assert_eq!(s.error_count, 0);
    assert_eq!(s.min_response_ms, None);
    assert_eq!(s.max_response_ms, None);
    assert_eq!(s.avg_response_ms, None);
}

#[test]
fn single_slot_query() {
    let mut mc = MetricsCollector::new(5);
    mc.record(3, 42.0, false);
    mc.record(3, 58.0, true);

    let s = mc.query(3, 3);
    assert_eq!(s.total_requests, 2);
    assert_eq!(s.error_count, 1);
    assert_eq!(s.min_response_ms, Some(42.0));
    assert_eq!(s.max_response_ms, Some(58.0));
    assert!((s.avg_response_ms.unwrap() - 50.0).abs() < 1e-9);

    // Neighbouring slots should not be affected
    assert_eq!(mc.query(2, 2).total_requests, 0);
    assert_eq!(mc.query(4, 4).total_requests, 0);
}

#[test]
fn min_max_aggregated_correctly_across_slots() {
    let mut mc = MetricsCollector::new(5);
    mc.record(0, 30.0, false);
    mc.record(1, 10.0, false); // global min
    mc.record(2, 50.0, false); // global max
    mc.record(3, 20.0, false);

    let s = mc.query(0, 3);
    assert_eq!(s.min_response_ms, Some(10.0));
    assert_eq!(s.max_response_ms, Some(50.0));
}

#[test]
fn two_time_windows_independent() {
    let mut mc = MetricsCollector::new(60);

    // Slots 0–29: healthy
    for slot in 0..30 {
        for _ in 0..10 {
            mc.record(slot, 50.0, false);
        }
    }
    // Slots 30–59: degraded (all errors, 10x slower)
    for slot in 30..60 {
        for _ in 0..10 {
            mc.record(slot, 500.0, true);
        }
    }

    let healthy = mc.query(0, 29);
    assert_eq!(healthy.total_requests, 300);
    assert_eq!(healthy.error_count, 0);
    assert_eq!(healthy.avg_response_ms, Some(50.0));

    let degraded = mc.query(30, 59);
    assert_eq!(degraded.total_requests, 300);
    assert_eq!(degraded.error_count, 300);
    assert_eq!(degraded.avg_response_ms, Some(500.0));

    let all = mc.query(0, 59);
    assert_eq!(all.total_requests, 600);
    assert!((all.error_rate - 0.5).abs() < 1e-9);
    assert_eq!(all.min_response_ms, Some(50.0));
    assert_eq!(all.max_response_ms, Some(500.0));
}

#[test]
fn error_rate_is_zero_when_no_errors() {
    let mut mc = MetricsCollector::new(5);
    mc.record(0, 100.0, false);
    mc.record(1, 200.0, false);
    let s = mc.query(0, 4);
    assert_eq!(s.error_rate, 0.0);
    assert_eq!(s.error_count, 0);
}

#[test]
fn error_rate_is_one_when_all_errors() {
    let mut mc = MetricsCollector::new(3);
    mc.record(0, 10.0, true);
    mc.record(1, 20.0, true);
    mc.record(2, 30.0, true);
    let s = mc.query(0, 2);
    assert!((s.error_rate - 1.0).abs() < 1e-9);
}
