use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberColorData {
	pub user_id: String,
	pub profile_picture_url: String,
	pub cached_color: Option<String>,
	pub cached_image: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSaveConfig {
	pub save_type: String,
	pub save_server: String,
	pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ImageTask {
	GenerateServerImage {
		guild_id: String,
		guild_name: String,
		guild_icon_url: String,
		image_type: String,
		members: Vec<MemberColorData>,
		blacklist: Vec<String>,
		image_save_config: ImageSaveConfig,
	},
	CalculateUserColor {
		user_id: String,
		profile_picture_url: String,
	},
}
