/// Items scoring at most this are dropped from fuzzy search results.
const MIN_SIMILARITY: f64 = 0.5;

/// Keeps the items whose `score` (0..=1, e.g. Jaro-Winkler) exceeds [`MIN_SIMILARITY`],
/// most similar first.
pub fn rank_by_similarity<T>(items: Vec<T>, score: impl Fn(&T) -> f64) -> Vec<T> {
    let mut scored = items
        .into_iter()
        .map(|item| (score(&item), item))
        .filter(|(similarity, _)| *similarity > MIN_SIMILARITY)
        .collect::<Vec<_>>();
    scored.sort_by(|(a, _), (b, _)| b.total_cmp(a));
    scored.into_iter().map(|(_, item)| item).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_and_sorts_descending() {
        let ranked = rank_by_similarity(vec![0.6, 0.2, 0.9, 0.5], |x| *x);
        assert_eq!(ranked, vec![0.9, 0.6]);
    }
}
