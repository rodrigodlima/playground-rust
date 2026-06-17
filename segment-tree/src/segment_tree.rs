
pub struct SegmentTree<T: Clone> {
    tree: Vec<T>,
    n: usize,
    identity: T,
    merge: fn(&T, &T) -> T,
}


