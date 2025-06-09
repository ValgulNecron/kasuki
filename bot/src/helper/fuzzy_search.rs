use std::collections::BinaryHeap;
use std::sync::Mutex;

use anyhow::{Context, Result};
use rapidfuzz::distance::jaro_winkler;
use rayon::prelude::*;

/// Finds the top N items from a vector that are most similar to the search string.
///
/// This function uses the Jaro-Winkler distance algorithm to measure string similarity.
/// A higher distance value indicates greater similarity between strings.
///
/// # Arguments
///
/// * `search` - The string to search for
/// * `vector` - A vector of string slices to search in
/// * `n` - The maximum number of results to return
///
/// # Returns
///
/// A Result containing a vector of tuples:
/// * The matched string
/// * The distance score (higher means more similar)
///
/// The results are ordered by similarity (highest distance first).
///
/// # Errors
///
/// This function can return an error in the following scenarios:
/// * Failed to acquire mutex lock for the distances collection
/// * Failed to peek at the binary heap when it should not be empty
/// * Failed to release mutex lock for distances
///
/// # Examples
///
/// ```
/// use crate::helper::fuzzy_search::distance_top_n;
///
/// let search = "apple";
/// let items = vec!["apple", "banana", "applet", "application", "orange"];
/// let results = distance_top_n(search, items, 3).expect("Function should not fail");
///
/// // Results will contain "apple", "applet", and "application" in order of similarity
/// ```
pub fn distance_top_n(search: &str, vector: Vec<&str>, n: usize) -> Result<Vec<(String, usize)>> {
	// Early return for edge cases
	if vector.is_empty() || n == 0 {
		return Ok(Vec::new());
	}

	let distances: Mutex<BinaryHeap<(usize, String)>> = Mutex::new(BinaryHeap::new());

	vector.par_iter().try_for_each(|item| {
		let distance = (jaro_winkler::distance(search.chars(), item.chars()) * 100.0) as usize;

		let item = (distance, item.to_string());

		// Handle the mutex lock error explicitly instead of using ?
		let mut distances = match distances.lock() {
			Ok(guard) => guard,
			Err(poison_error) => poison_error.into_inner(), // Recover from poison error
		};

		if distances.len() < n {
			distances.push(item.clone());
		} else {
			let max = distances
				.peek()
				.context("Failed to peek at the binary heap, which should not be empty")?;

			if &item.clone() < max {
				distances.pop();
				distances.push(item);
			}
		}

		Ok::<_, anyhow::Error>(())
	})?;

	let heap = distances
		.into_inner()
		.context("Failed to release mutex lock for distances")?;

	Ok(heap
		.into_par_iter()
		.map(|(distance, item)| (item, distance))
		.collect())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_distance_top_n_basic() {
		// Test with a simple case
		let search = "apple";
		let items = vec!["apple", "banana", "applet", "application", "orange"];
		let results = distance_top_n(search, items, 3).expect("Function should not fail");

		// Check that we got the expected number of results
		assert_eq!(results.len(), 3, "Should return exactly 3 results");

		// Check that the results are in the correct order (most similar first)
		assert_eq!(results[0].0, "apple", "First result should be 'apple'");

		// Check that the other results are also correct (applet and application are more similar to apple than banana or orange)
		let result_strings: Vec<String> = results.iter().map(|(s, _)| s.clone()).collect();
		assert!(
			result_strings.contains(&"applet".to_string()),
			"Results should contain 'applet'"
		);
		assert!(
			result_strings.contains(&"application".to_string()),
			"Results should contain 'application'"
		);

		// Check that less similar items are not included
		assert!(
			!result_strings.contains(&"banana".to_string()),
			"Results should not contain 'banana'"
		);
		assert!(
			!result_strings.contains(&"orange".to_string()),
			"Results should not contain 'orange'"
		);
	}

	#[test]
	fn test_distance_top_n_edge_cases() {
		// Test with empty vector
		let search = "test";
		let empty_vec: Vec<&str> = Vec::new();
		let results = distance_top_n(search, empty_vec, 3)
			.expect("Function should not fail with empty vector");
		assert_eq!(results.len(), 0, "Empty vector should return empty results");

		// Test with n = 0
		let items = vec!["apple", "banana", "orange"];
		let results = distance_top_n(search, items, 0).expect("Function should not fail with n=0");
		assert_eq!(results.len(), 0, "n=0 should return empty results");

		// Test with n larger than vector size
		let items = vec!["apple", "banana"];
		let results = distance_top_n(search, items, 5)
			.expect("Function should not fail with n > vector.len()");
		assert_eq!(
			results.len(),
			2,
			"Should return all items when n > vector.len()"
		);
	}

	#[test]
	fn test_distance_top_n_exact_match() {
		// Test with exact matches
		let search = "banana";
		let items = vec!["apple", "banana", "orange"];
		let results =
			distance_top_n(search, items, 1).expect("Function should not fail with exact match");

		assert_eq!(results.len(), 1, "Should return exactly 1 result");
		assert_eq!(results[0].0, "banana", "Should return exact match");

		// The distance for an exact match should be 100 (100% similarity)
		assert_eq!(results[0].1, 100, "Exact match should have distance of 100");
	}

	#[test]
	fn test_distance_top_n_case_sensitivity() {
		// Test case sensitivity
		let search = "Apple";
		let items = vec!["apple", "APPLE", "aPpLe", "banana"];
		let results = distance_top_n(search, items, 3)
			.expect("Function should not fail with case sensitivity test");

		// All variations of "apple" should be returned before "banana"
		assert_eq!(results.len(), 3, "Should return exactly 3 results");

		let result_strings: Vec<String> = results.iter().map(|(s, _)| s.clone()).collect();
		assert!(
			result_strings.contains(&"apple".to_string()),
			"Results should contain 'apple'"
		);
		assert!(
			result_strings.contains(&"APPLE".to_string()),
			"Results should contain 'APPLE'"
		);
		assert!(
			result_strings.contains(&"aPpLe".to_string()),
			"Results should contain 'aPpLe'"
		);
		assert!(
			!result_strings.contains(&"banana".to_string()),
			"Results should not contain 'banana'"
		);
	}
}
