/// A single field to display (label, value, inline).
pub type DisplayField = (String, String, bool);

/// Result of a character lookup.
pub struct CharacterResult {
	pub name: String,
	pub id: String,
	pub description: Option<String>,
	pub fields: Vec<DisplayField>,
	pub image_url: Option<String>,
	pub url: String,
}

/// Result of a VN game lookup.
pub struct GameResult {
	pub title: String,
	pub id: String,
	pub description: Option<String>,
	pub fields: Vec<DisplayField>,
	pub image_url: Option<String>,
	pub url: String,
}

/// Result of a VNDB stats lookup.
pub struct StatsResult {
	pub title: String,
	pub fields: Vec<DisplayField>,
}

/// Result of a VNDB user lookup.
pub struct UserResult {
	pub title: String,
	pub fields: Vec<DisplayField>,
}

/// Result of a VN staff lookup.
pub struct StaffResult {
	pub name: String,
	pub id: String,
	pub description: Option<String>,
	pub fields: Vec<DisplayField>,
	pub url: String,
}

/// Result of a VN producer lookup.
pub struct ProducerResult {
	pub name: String,
	pub id: String,
	pub description: Option<String>,
	pub fields: Vec<DisplayField>,
	pub url: String,
}
