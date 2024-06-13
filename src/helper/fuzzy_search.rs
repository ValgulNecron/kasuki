use rapidfuzz::distance::jaro_winkler;
use rayon::prelude::*;
use std::collections::BinaryHeap;
use std::sync::Mutex;

pub fn distance_top_n(search: &str, vector: Vec<&str>, n: usize) -> Vec<(String, usize)> {
    let distances: Mutex<BinaryHeap<(usize, String)>> = Mutex::new(BinaryHeap::new());

    vector.par_iter().for_each(|item| {
        let distance = (jaro_winkler::distance(search.chars(), item.chars()) * 100.0) as usize;
        let item = (distance, item.to_string());
        let mut distances = distances.lock().unwrap();
        if distances.len() < n {
            distances.push(item.clone());
        } else {
            let max = distances.peek().unwrap();
            if &item.clone() < max {
                distances.pop();
                distances.push(item);
            }
        }
    });

    distances
        .into_inner()
        .unwrap()
        .into_par_iter()
        .map(|(distance, item)| (item, distance))
        .collect()
}
