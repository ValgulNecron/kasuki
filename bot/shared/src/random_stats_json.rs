use serde::{Deserialize, Serialize};

/// Represents the random statistics of anime and manga.
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct RandomStat {
	/// The last page of anime statistics.
	pub anime_last_page: i32,
	/// The last page of manga statistics.
	pub manga_last_page: i32,
}

impl Default for RandomStat {
	/// Returns a default `RandomStat` with `anime_last_page` set to 1796 and `manga_last_page` set to 1796.
	fn default() -> Self {
		Self {
			anime_last_page: 1796,
			manga_last_page: 1796,
		}
	}
}
