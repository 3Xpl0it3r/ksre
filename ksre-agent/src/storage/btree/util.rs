pub(super) fn bytes_compare(a: &[u8], b: &[u8]) -> std::cmp::Ordering {
    for (ai, bi) in a.iter().zip(b.iter()) {
        match ai.cmp(bi) {
            std::cmp::Ordering::Equal => continue,
            ord => return ord,
        }
    }
    a.len().cmp(&b.len())
}
