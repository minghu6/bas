use std::cmp::Ordering;


////////////////////////////////////////////////////////////////////////////////
//// Macro



////////////////////////////////////////////////////////////////////////////////
//// Structure

/// KeyValue Pair
#[derive(Debug, Clone, Copy)]
pub struct KVEntry<K, V>(pub K, pub V);


#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct TOEntry2<T1, T2>(pub T1, pub T2);


////////////////////////////////////////////////////////////////////////////////
//// Implementation

impl<K, V> KVEntry<K, V> {
    pub fn drain(self) -> (K, V) {
        (self.0, self.1)
    }
}

impl<K: PartialEq, V> PartialEq for KVEntry<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<K: PartialOrd, V> PartialOrd for KVEntry<K, V> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<K: Eq, V> Eq for KVEntry<K, V> {
}

impl<K: Ord, V> Ord for KVEntry<K, V> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(&other).unwrap()
    }
}

impl<K, V> Into<(K, V)> for KVEntry<K, V> {
    fn into(self) -> (K, V) {
        (self.0, self.1)
    }
}



#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_kventry() {
        use std::collections::BinaryHeap;
        use std::cmp::Reverse;

        let mut heap = BinaryHeap::new();

        heap.push(Reverse(KVEntry(2, 3)));
        heap.push(Reverse(KVEntry(1, 1)));
        heap.push(Reverse(KVEntry(4, 16)));
        heap.push(Reverse(KVEntry(2, 1)));


        assert_eq!(heap.pop(), Some(Reverse(KVEntry(1, 1))));
        assert_eq!(heap.pop(), Some(Reverse(KVEntry(2, 3))));
        assert_eq!(heap.pop(), Some(Reverse(KVEntry(2, 1))));
        assert_eq!(heap.pop(), Some(Reverse(KVEntry(4, 16))));

        assert_eq!(heap.pop(), None);

    }


    #[test]
    fn test_toentry() {
        assert!((3, 2) > (3, 1));

        assert!((3, 2) > (3, 1));
        assert!((3, 2) < (3, 3));
        assert!((3, 3) == (3, 3));

        assert!((2, 3) < (3, 2));
        assert!((2, 2) < (3, 3));
    }


}
