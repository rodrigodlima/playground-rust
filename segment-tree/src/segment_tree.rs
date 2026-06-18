
pub struct SegmentTree<T: Clone> {
    tree: Vec<T>,
    n: usize,
    identity: T,
    merge: fn(&T, &T) -> T,
}

impl <T: Clone> SegmentTree<T> {
    new(n, identity, merge)
    
}


