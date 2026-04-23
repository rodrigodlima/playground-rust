use crate::SegmentTree;

fn add_f64(a: &f64, b: &f64) -> f64 { *a + *b }
fn add_u64(a: &u64, b: &u64) -> u64 { *a + *b }
fn min_f64(a: &f64, b: &f64) -> f64 { f64::min(*a, *b) }
fn max_f64(a: &f64, b: &f64) -> f64 { f64::max(*a, *b) }

/// Per-slot raw accumulator (not exposed to callers directly).
#[derive(Debug, Clone)]
pub struct SlotMetrics {
    pub count: u64,
    pub error_count: u64,
    pub sum_ms: f64,
    pub min_ms: f64,
    pub max_ms: f64,
}

impl Default for SlotMetrics {
    fn default() -> Self {
        SlotMetrics {
            count: 0,
            error_count: 0,
            sum_ms: 0.0,
            min_ms: f64::INFINITY,
            max_ms: f64::NEG_INFINITY,
        }
    }
}

/// Result of a range query over one or more slots.
#[derive(Debug)]
pub struct MetricsSummary {
    pub total_requests: u64,
    pub error_count: u64,
    /// Fraction of requests that were errors. 0.0 when no requests.
    pub error_rate: f64,
    /// None when no requests were recorded in the queried range.
    pub min_response_ms: Option<f64>,
    pub max_response_ms: Option<f64>,
    pub avg_response_ms: Option<f64>,
}

/// Collects request metrics over fixed time slots and answers range queries
/// in O(log n) using five internal segment trees (count, errors, sum, min, max).
///
/// Typical use: each slot = one minute (or hour). Record every request as it
/// arrives, then query arbitrary windows for dashboards or alerting.
pub struct MetricsCollector {
    num_slots: usize,
    raw: Vec<SlotMetrics>,
    count_tree: SegmentTree<u64, fn(&u64, &u64) -> u64>,
    error_tree: SegmentTree<u64, fn(&u64, &u64) -> u64>,
    sum_tree: SegmentTree<f64, fn(&f64, &f64) -> f64>,
    min_tree: SegmentTree<f64, fn(&f64, &f64) -> f64>,
    max_tree: SegmentTree<f64, fn(&f64, &f64) -> f64>,
}

impl MetricsCollector {
    pub fn new(num_slots: usize) -> Self {
        MetricsCollector {
            num_slots,
            raw: vec![SlotMetrics::default(); num_slots],
            count_tree: SegmentTree::new(num_slots, 0u64, add_u64),
            error_tree: SegmentTree::new(num_slots, 0u64, add_u64),
            sum_tree: SegmentTree::new(num_slots, 0.0f64, add_f64),
            min_tree: SegmentTree::new(num_slots, f64::INFINITY, min_f64),
            max_tree: SegmentTree::new(num_slots, f64::NEG_INFINITY, max_f64),
        }
    }

    /// Records one request in the given slot. O(log n).
    pub fn record(&mut self, slot: usize, response_time_ms: f64, is_error: bool) {
        assert!(slot < self.num_slots, "slot {slot} out of bounds");

        let s = &mut self.raw[slot];
        s.count += 1;
        s.sum_ms += response_time_ms;
        s.min_ms = f64::min(s.min_ms, response_time_ms);
        s.max_ms = f64::max(s.max_ms, response_time_ms);
        if is_error {
            s.error_count += 1;
        }

        // Capture values before releasing the mutable borrow on `raw`.
        let (count, error_count, sum_ms, min_ms, max_ms) =
            (s.count, s.error_count, s.sum_ms, s.min_ms, s.max_ms);

        self.count_tree.update(slot, count);
        self.error_tree.update(slot, error_count);
        self.sum_tree.update(slot, sum_ms);
        self.min_tree.update(slot, min_ms);
        self.max_tree.update(slot, max_ms);
    }

    /// Aggregates metrics across all slots in `[from_slot, to_slot]`. O(log n).
    pub fn query(&self, from_slot: usize, to_slot: usize) -> MetricsSummary {
        assert!(
            from_slot <= to_slot && to_slot < self.num_slots,
            "invalid slot range [{from_slot}, {to_slot}] for {} slots",
            self.num_slots
        );

        let total_requests = self.count_tree.query(from_slot, to_slot);
        let error_count = self.error_tree.query(from_slot, to_slot);
        let sum_ms = self.sum_tree.query(from_slot, to_slot);
        let min_ms = self.min_tree.query(from_slot, to_slot);
        let max_ms = self.max_tree.query(from_slot, to_slot);

        let (min_response_ms, max_response_ms, avg_response_ms) = if total_requests > 0 {
            (Some(min_ms), Some(max_ms), Some(sum_ms / total_requests as f64))
        } else {
            (None, None, None)
        };

        let error_rate = if total_requests > 0 {
            error_count as f64 / total_requests as f64
        } else {
            0.0
        };

        MetricsSummary {
            total_requests,
            error_count,
            error_rate,
            min_response_ms,
            max_response_ms,
            avg_response_ms,
        }
    }

    pub fn slot_count(&self) -> usize {
        self.num_slots
    }

    pub fn raw_slot(&self, slot: usize) -> &SlotMetrics {
        &self.raw[slot]
    }
}
