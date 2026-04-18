use std::collections::{HashMap, HashSet};

struct GameEntry {
	name: String,
	name_lower: String,
	app_id: u32,
}

/// A pre-built search index for Steam games that supports fast multi-word prefix matching
/// with substring fallback.
///
/// Build cost: O(n * avg_words) — done once when the game list is loaded.
/// Query cost: O(k * log(w) + m) for word-prefix, with O(n) substring fallback only when
/// the fast path returns fewer results than the limit.
pub struct SteamGameIndex {
	games: Vec<GameEntry>,
	/// Sorted by word. Each entry is (lowercase_word, index into `games`).
	word_entries: Vec<(Box<str>, u32)>,
}

impl Default for SteamGameIndex {
	fn default() -> Self {
		Self {
			games: Vec::new(),
			word_entries: Vec::new(),
		}
	}
}

/// Returns true if the name looks like a junk/placeholder Steam entry.
fn is_junk_entry(name: &str) -> bool {
	let trimmed = name.trim();
	if trimmed.is_empty() {
		return true;
	}
	let lower = trimmed.to_ascii_lowercase();
	if lower == "undefined" {
		return true;
	}
	if lower.starts_with("steamapp") || lower.starts_with("valvetestapp") {
		return true;
	}
	false
}

impl SteamGameIndex {
	pub fn from_map(map: HashMap<String, u32>) -> Self {
		let games: Vec<GameEntry> = map
			.into_iter()
			.filter(|(name, _)| !is_junk_entry(name))
			.map(|(name, app_id)| {
				let name_lower = name.to_ascii_lowercase();
				GameEntry {
					name,
					name_lower,
					app_id,
				}
			})
			.collect();

		let mut word_entries: Vec<(Box<str>, u32)> = Vec::with_capacity(games.len() * 3);
		for (idx, game) in games.iter().enumerate() {
			for word in game.name_lower.split_whitespace() {
				word_entries.push((word.into(), idx as u32));
			}
		}
		word_entries.sort_unstable_by(|a, b| a.0.cmp(&b.0));

		Self {
			games,
			word_entries,
		}
	}

	pub fn is_empty(&self) -> bool {
		self.games.is_empty()
	}

	pub fn len(&self) -> usize {
		self.games.len()
	}

	/// Convert back to a HashMap for serialization/caching.
	pub fn to_map(&self) -> HashMap<String, u32> {
		self.games
			.iter()
			.map(|g| (g.name.clone(), g.app_id))
			.collect()
	}

	/// Search for games matching the query. Returns up to `limit` results as `(name, app_id)`.
	///
	/// Uses a two-phase strategy:
	/// 1. **Word-prefix matching** (fast, O(log n) per query word) — finds games whose words
	///    start with each query word. Handles multi-word queries via intersection.
	/// 2. **Substring fallback** (O(n) scan) — if the fast path didn't fill `limit` results,
	///    scans all games for substring matches. This handles queries like "craft" matching
	///    "Minecraft" where the query isn't at a word boundary.
	pub fn search(&self, query: &str, limit: usize) -> Vec<(&str, u32)> {
		if query.is_empty() || limit == 0 || self.games.is_empty() {
			return Vec::new();
		}

		let query_lower = query.to_ascii_lowercase();
		let query_words: Vec<&str> = query_lower.split_whitespace().collect();

		if query_words.is_empty() {
			return Vec::new();
		}

		let mut results = self.search_by_word_prefix(&query_lower, &query_words, limit);

		if results.len() < limit {
			let already_found: HashSet<u32> = results.iter().map(|(_, id)| *id).collect();
			let remaining = limit - results.len();
			let extra = self.search_by_substring(&query_lower, &query_words, remaining, &already_found);
			results.extend(extra);
		}

		results
	}

	/// Fast path: find games where each query word matches a word-prefix in the game name.
	fn search_by_word_prefix<'a>(
		&'a self, query_lower: &str, query_words: &[&str], limit: usize,
	) -> Vec<(&'a str, u32)> {
		let mut candidate_sets: Vec<Vec<u32>> = Vec::with_capacity(query_words.len());

		for qw in query_words {
			let indices = self.find_word_prefix_matches(qw);
			if indices.is_empty() {
				return Vec::new();
			}
			candidate_sets.push(indices);
		}

		let mut candidates = candidate_sets.remove(0);
		candidates.sort_unstable();
		candidates.dedup();

		for set in &candidate_sets {
			let mut sorted_set = set.clone();
			sorted_set.sort_unstable();
			sorted_set.dedup();
			candidates = intersect_sorted(&candidates, &sorted_set);
		}

		if candidates.is_empty() {
			return Vec::new();
		}

		self.score_and_rank(candidates, query_lower, query_words, limit)
	}

	/// Slow path: scan all games for substring matches.
	/// Only called when word-prefix didn't produce enough results.
	fn search_by_substring<'a>(
		&'a self, query_lower: &str, query_words: &[&str], limit: usize,
		exclude: &HashSet<u32>,
	) -> Vec<(&'a str, u32)> {
		let mut candidates: Vec<u32> = Vec::new();

		for (idx, game) in self.games.iter().enumerate() {
			let idx = idx as u32;
			if exclude.contains(&game.app_id) {
				continue;
			}

			let all_match = query_words
				.iter()
				.all(|qw| game.name_lower.contains(qw));

			if all_match {
				candidates.push(idx);
			}
		}

		if candidates.is_empty() {
			return Vec::new();
		}

		self.score_and_rank(candidates, query_lower, query_words, limit)
	}

	/// Score candidates and return the top `limit` results.
	fn score_and_rank<'a>(
		&'a self, candidates: Vec<u32>, query_lower: &str, query_words: &[&str], limit: usize,
	) -> Vec<(&'a str, u32)> {
		let mut scored: Vec<(u32, i64)> = candidates
			.into_iter()
			.map(|idx| {
				let game = &self.games[idx as usize];
				let score = Self::score_match(&game.name_lower, query_lower, query_words);
				(idx, score)
			})
			.collect();

		scored.sort_unstable_by(|a, b| {
			b.1.cmp(&a.1).then_with(|| {
				let a_len = self.games[a.0 as usize].name.len();
				let b_len = self.games[b.0 as usize].name.len();
				a_len.cmp(&b_len)
			})
		});

		scored
			.into_iter()
			.take(limit)
			.map(|(idx, _)| {
				let game = &self.games[idx as usize];
				(game.name.as_str(), game.app_id)
			})
			.collect()
	}

	/// Binary search the sorted word index for all words starting with `prefix`.
	fn find_word_prefix_matches(&self, prefix: &str) -> Vec<u32> {
		let start = self
			.word_entries
			.partition_point(|(w, _)| w.as_ref() < prefix);

		let mut results = Vec::new();
		for (word, idx) in &self.word_entries[start..] {
			if word.starts_with(prefix) {
				results.push(*idx);
			} else {
				break;
			}
		}
		results
	}

	fn score_match(name_lower: &str, query_lower: &str, query_words: &[&str]) -> i64 {
		let mut score: i64 = 0;

		if name_lower == query_lower {
			score += 10000;
		} else if name_lower.starts_with(query_lower) {
			score += 5000;
		} else if name_lower.contains(query_lower) {
			score += 2000;
		}

		let name_words: Vec<&str> = name_lower.split_whitespace().collect();
		for qw in query_words {
			if name_words.contains(qw) {
				score += 500;
			}
		}

		score -= name_lower.len() as i64;

		score
	}
}

fn intersect_sorted(a: &[u32], b: &[u32]) -> Vec<u32> {
	let mut result = Vec::new();
	let (mut i, mut j) = (0, 0);
	while i < a.len() && j < b.len() {
		match a[i].cmp(&b[j]) {
			std::cmp::Ordering::Less => i += 1,
			std::cmp::Ordering::Greater => j += 1,
			std::cmp::Ordering::Equal => {
				result.push(a[i]);
				i += 1;
				j += 1;
			},
		}
	}
	result
}

#[cfg(test)]
mod tests {
	use super::*;

	fn make_index(games: &[(&str, u32)]) -> SteamGameIndex {
		let map: HashMap<String, u32> = games
			.iter()
			.map(|(name, id)| (name.to_string(), *id))
			.collect();
		SteamGameIndex::from_map(map)
	}

	#[test]
	fn test_exact_match() {
		let index = make_index(&[
			("Counter-Strike 2", 730),
			("Counter-Strike Source", 240),
			("Half-Life 2", 220),
		]);
		let results = index.search("Counter-Strike 2", 5);
		assert!(!results.is_empty());
		assert_eq!(results[0].0, "Counter-Strike 2");
		assert_eq!(results[0].1, 730);
	}

	#[test]
	fn test_prefix_match() {
		let index = make_index(&[
			("Dark Souls III", 374320),
			("Dark Souls II", 236430),
			("Dark Souls Remastered", 570940),
			("Darksiders", 50620),
		]);
		let results = index.search("dark s", 10);
		assert_eq!(results.len(), 4);
		let top3: Vec<&str> = results[..3].iter().map(|(n, _)| *n).collect();
		for name in &top3 {
			assert!(
				name.to_ascii_lowercase().contains("dark souls"),
				"Top results should be Dark Souls games, got: {}",
				name
			);
		}
	}

	#[test]
	fn test_multi_word_query() {
		let index = make_index(&[
			("Grand Theft Auto V", 271590),
			("Grand Theft Auto IV", 12210),
			("Grand Prix Racing", 99999),
			("Auto Chess", 88888),
		]);
		let results = index.search("grand theft", 10);
		assert_eq!(results.len(), 2);
		for (name, _) in &results {
			let lower = name.to_ascii_lowercase();
			assert!(lower.contains("grand") && lower.contains("theft"));
		}
	}

	#[test]
	fn test_partial_word_match() {
		let index = make_index(&[
			("The Elder Scrolls V Skyrim", 72850),
			("The Elder Scrolls Online", 306130),
			("Elder Ring", 99999),
		]);
		let results = index.search("elder scr", 10);
		assert_eq!(results.len(), 2);
	}

	#[test]
	fn test_empty_query() {
		let index = make_index(&[("Test Game", 1)]);
		assert!(index.search("", 10).is_empty());
	}

	#[test]
	fn test_no_match() {
		let index = make_index(&[("Test Game", 1)]);
		assert!(index.search("xyz", 10).is_empty());
	}

	#[test]
	fn test_empty_index() {
		let index = SteamGameIndex::default();
		assert!(index.search("test", 10).is_empty());
		assert!(index.is_empty());
	}

	#[test]
	fn test_limit() {
		let index = make_index(&[
			("Test Game 1", 1),
			("Test Game 2", 2),
			("Test Game 3", 3),
			("Test Game 4", 4),
		]);
		let results = index.search("test", 2);
		assert_eq!(results.len(), 2);
	}

	#[test]
	fn test_case_insensitive() {
		let index = make_index(&[("DARK SOULS III", 374320)]);
		let results = index.search("dark souls", 5);
		assert_eq!(results.len(), 1);
		assert_eq!(results[0].0, "DARK SOULS III");
	}

	#[test]
	fn test_to_map_roundtrip() {
		let original: HashMap<String, u32> = [
			("Game A".to_string(), 1u32),
			("Game B".to_string(), 2u32),
		]
		.into_iter()
		.collect();
		let index = SteamGameIndex::from_map(original.clone());
		let recovered = index.to_map();
		assert_eq!(recovered, original);
	}

	#[test]
	fn test_scoring_exact_over_prefix() {
		let index = make_index(&[
			("Portal", 400),
			("Portal 2", 620),
			("Portal Knights", 374040),
		]);
		let results = index.search("portal", 10);
		assert!(!results.is_empty());
		assert_eq!(results[0].0, "Portal");
	}

	#[test]
	fn test_scoring_shorter_name_preferred() {
		let index = make_index(&[
			("Rust", 252490),
			("Rusty Lake Hotel", 123456),
			("Rusty Lake Paradise", 654321),
		]);
		let results = index.search("rust", 10);
		assert!(!results.is_empty());
		assert_eq!(results[0].0, "Rust");
	}

	#[test]
	fn test_single_character_query() {
		let index = make_index(&[("A Short Hike", 1), ("ABZU", 2), ("Braid", 3)]);
		let results = index.search("a", 10);
		assert!(results.len() >= 2);
		let names: Vec<&str> = results.iter().map(|(n, _)| *n).collect();
		assert!(names.contains(&"ABZU"));
		assert!(names.contains(&"A Short Hike"));
	}

	#[test]
	fn test_substring_craft_matches_minecraft() {
		let index = make_index(&[
			("Minecraft", 1),
			("Minecraft Dungeons", 2),
			("Craftopia", 3),
			("Unrelated Game", 4),
		]);
		let results = index.search("craft", 10);
		assert!(results.len() >= 2);
		let names: Vec<&str> = results.iter().map(|(n, _)| *n).collect();
		assert!(names.contains(&"Minecraft"));
		assert!(names.contains(&"Craftopia"));
	}

	#[test]
	fn test_substring_multi_word() {
		let index = make_index(&[
			("Minecraft", 1),
			("Minecraft Dungeons", 2),
			("Warcraft III", 3),
		]);
		let results = index.search("mine craft", 10);
		assert!(!results.is_empty());
		for (name, _) in &results {
			let lower = name.to_ascii_lowercase();
			assert!(lower.contains("mine") && lower.contains("craft"));
		}
	}

	#[test]
	fn test_substring_mid_word() {
		let index = make_index(&[
			("Terraria", 1),
			("Undertale", 2),
			("Terra Nil", 3),
		]);
		let results = index.search("erra", 10);
		assert!(results.len() >= 2);
		let names: Vec<&str> = results.iter().map(|(n, _)| *n).collect();
		assert!(names.contains(&"Terraria"));
		assert!(names.contains(&"Terra Nil"));
	}

	#[test]
	fn test_filters_empty_names() {
		let index = make_index(&[("", 1), ("  ", 2), ("Real Game", 3)]);
		assert_eq!(index.len(), 1);
		let results = index.search("real", 10);
		assert_eq!(results.len(), 1);
		assert_eq!(results[0].0, "Real Game");
	}

	#[test]
	fn test_filters_undefined() {
		let index = make_index(&[("Undefined", 1), ("undefined", 2), ("Real Game", 3)]);
		assert_eq!(index.len(), 1);
	}

	#[test]
	fn test_filters_test_apps() {
		let index = make_index(&[
			("SteamApp 12345", 1),
			("ValveTestApp999", 2),
			("Real Game", 3),
		]);
		assert_eq!(index.len(), 1);
		assert_eq!(index.search("real", 10)[0].0, "Real Game");
	}

	#[test]
	fn test_keeps_numeric_names() {
		let index = make_index(&[("1942", 1), ("2048", 2), ("Real Game", 3)]);
		assert_eq!(index.len(), 3);
		let results = index.search("1942", 10);
		assert_eq!(results.len(), 1);
		assert_eq!(results[0].0, "1942");
	}

	#[test]
	fn test_word_prefix_preferred_over_substring() {
		let index = make_index(&[
			("Craftopia", 1),
			("Minecraft", 2),
			("Craft the World", 3),
		]);
		let results = index.search("craft", 10);
		assert_eq!(results.len(), 3);
		assert!(
			results[0].0 == "Craft the World" || results[0].0 == "Craftopia",
			"First result should be a word-prefix match, got: {}",
			results[0].0
		);
	}
}
