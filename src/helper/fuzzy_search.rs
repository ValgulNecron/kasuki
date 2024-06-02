use rapidfuzz::distance::damerau_levenshtein;

pub fn distance_top_n(search: &str, vector: Vec<&str>, n: usize) -> Vec<(String, usize)> {
    let mut distances = vec![];

    for item in vector {
        let distance = damerau_levenshtein::distance(search.chars(), item.chars());
        distances.push((item.to_string(), distance));
    }

    distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    distances.reverse();
    distances.truncate(n);

    distances
}
