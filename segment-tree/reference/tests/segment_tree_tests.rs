use segment_tree::SegmentTree;

#[test]
fn sum_from_data() {
    let data = vec![1_i64, 3, 5, 7, 9, 11];
    let tree = SegmentTree::from_data(&data, 0, |a, b| *a + *b);
    assert_eq!(tree.query(0, 5), 36);
    assert_eq!(tree.query(0, 0), 1);
    assert_eq!(tree.query(2, 4), 21);
    assert_eq!(tree.query(1, 3), 15);
}

#[test]
fn sum_point_update() {
    let mut tree = SegmentTree::new(5, 0_i64, |a, b| *a + *b);
    for i in 0..5usize {
        tree.update(i, i as i64 + 1); // [1, 2, 3, 4, 5]
    }
    assert_eq!(tree.query(0, 4), 15);
    tree.update(2, 10); // [1, 2, 10, 4, 5]
    assert_eq!(tree.query(0, 4), 22);
    assert_eq!(tree.query(2, 2), 10);
}

#[test]
fn min_range_query() {
    let data = vec![3_i64, 1, 4, 1, 5, 9, 2, 6];
    let tree = SegmentTree::from_data(&data, i64::MAX, |a, b| (*a).min(*b));
    assert_eq!(tree.query(0, 7), 1);
    assert_eq!(tree.query(4, 7), 2);
    assert_eq!(tree.query(5, 5), 9);
}

#[test]
fn max_range_query() {
    let data = vec![3_i64, 1, 4, 1, 5, 9, 2, 6];
    let tree = SegmentTree::from_data(&data, i64::MIN, |a, b| (*a).max(*b));
    assert_eq!(tree.query(0, 7), 9);
    assert_eq!(tree.query(0, 2), 4);
    assert_eq!(tree.query(6, 7), 6);
}

#[test]
fn single_element() {
    let mut tree = SegmentTree::new(1, 0_i64, |a, b| *a + *b);
    tree.update(0, 42);
    assert_eq!(tree.query(0, 0), 42);
}

#[test]
fn full_range_equals_sum_of_parts() {
    let n = 8;
    let mut tree = SegmentTree::new(n, 0_u64, |a, b| *a + *b);
    for i in 0..n {
        tree.update(i, (i + 1) as u64);
    }
    let mid = n / 2 - 1;
    assert_eq!(
        tree.query(0, n - 1),
        tree.query(0, mid) + tree.query(mid + 1, n - 1)
    );
}

#[test]
fn repeated_updates_to_same_index() {
    let mut tree = SegmentTree::new(4, 0_i64, |a, b| *a + *b);
    tree.update(2, 10);
    assert_eq!(tree.query(0, 3), 10);
    tree.update(2, 20);
    assert_eq!(tree.query(0, 3), 20);
    tree.update(2, 0);
    assert_eq!(tree.query(0, 3), 0);
}
