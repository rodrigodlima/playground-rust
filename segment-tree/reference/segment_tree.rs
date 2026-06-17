/// Generic segment tree supporting any associative operation.
///
/// - `update(idx, val)` — O(log n) point update
/// - `query(l, r)`     — O(log n) range query (inclusive)
pub struct SegmentTree<T, F>
where
    T: Clone,
    F: Fn(&T, &T) -> T,
{
    n: usize,
    tree: Vec<T>,
    identity: T,
    combine: F,
}

impl<T, F> SegmentTree<T, F>
where
    T: Clone,
    F: Fn(&T, &T) -> T,
{
    /// Creates an empty tree of `n` slots, all initialised to `identity`.
    pub fn new(n: usize, identity: T, combine: F) -> Self {
        assert!(n > 0, "segment tree size must be > 0");
        let tree = vec![identity.clone(); 4 * n];
        SegmentTree { n, tree, identity, combine }
    }

    /// Builds a tree directly from a slice in O(n).
    pub fn from_data(data: &[T], identity: T, combine: F) -> Self {
        assert!(!data.is_empty(), "data must not be empty");
        let n = data.len();
        let tree = vec![identity.clone(); 4 * n];
        let mut st = SegmentTree { n, tree, identity, combine };
        st.build(data, 1, 0, n - 1);
        st
    }

    fn build(&mut self, data: &[T], node: usize, start: usize, end: usize) {
        if start == end {
            self.tree[node] = data[start].clone();
            return;
        }
        let mid = start + (end - start) / 2;
        self.build(data, 2 * node, start, mid);
        self.build(data, 2 * node + 1, mid + 1, end);
        self.merge_children(node);
    }

    /// Sets `tree[idx] = val` and propagates upward.
    pub fn update(&mut self, idx: usize, val: T) {
        assert!(idx < self.n, "index {idx} out of bounds (size: {})", self.n);
        self.update_inner(1, 0, self.n - 1, idx, val);
    }

    fn update_inner(&mut self, node: usize, start: usize, end: usize, idx: usize, val: T) {
        if start == end {
            self.tree[node] = val;
            return;
        }
        let mid = start + (end - start) / 2;
        if idx <= mid {
            self.update_inner(2 * node, start, mid, idx, val);
        } else {
            self.update_inner(2 * node + 1, mid + 1, end, idx, val);
        }
        self.merge_children(node);
    }

    /// Returns `combine` of all values in the inclusive range `[l, r]`.
    pub fn query(&self, l: usize, r: usize) -> T {
        assert!(l <= r, "invalid range: l={l} > r={r}");
        assert!(r < self.n, "r={r} out of bounds (size: {})", self.n);
        self.query_inner(1, 0, self.n - 1, l, r)
    }

    fn query_inner(&self, node: usize, start: usize, end: usize, l: usize, r: usize) -> T {
        if r < start || end < l {
            return self.identity.clone();
        }
        if l <= start && end <= r {
            return self.tree[node].clone();
        }
        let mid = start + (end - start) / 2;
        let left = self.query_inner(2 * node, start, mid, l, r);
        let right = self.query_inner(2 * node + 1, mid + 1, end, l, r);
        (self.combine)(&left, &right)
    }

    fn merge_children(&mut self, node: usize) {
        // Clone children first to avoid simultaneous borrow conflicts.
        let left = self.tree[2 * node].clone();
        let right = self.tree[2 * node + 1].clone();
        self.tree[node] = (self.combine)(&left, &right);
    }

    pub fn len(&self) -> usize {
        self.n
    }

    pub fn is_empty(&self) -> bool {
        self.n == 0
    }
}
