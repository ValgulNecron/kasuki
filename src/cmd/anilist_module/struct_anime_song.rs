#[derive(Debug, serde::Deserialize)]
pub struct Artist {
    #[serde(rename = "id")]
    pub id: i32,
    #[serde(rename = "names")]
    pub names: Vec<String>,
    #[serde(rename = "line_up_id")]
    pub line_up_id: i32,
    #[serde(rename = "groups")]
    pub groups: Option<Vec<Group>>,
    #[serde(rename = "members")]
    pub members: Option<Vec<Artist>>,
}

#[derive(Debug, serde::Deserialize)]
pub struct Group {
    #[serde(rename = "id")]
    pub id: i32,
    #[serde(rename = "names")]
    pub names: Vec<String>,
    #[serde(rename = "line_up_id")]
    pub line_up_id: Option<i32>,
    #[serde(rename = "groups")]
    pub groups: Option<Vec<Group>>,
    #[serde(rename = "members")]
    pub members: Option<Vec<Artist>>,
}

#[derive(Debug, serde::Deserialize)]
pub struct Composer {
    #[serde(rename = "id")]
    pub id: i32,
    #[serde(rename = "names")]
    pub names: Vec<String>,
    #[serde(rename = "line_up_id")]
    pub line_up_id: Option<i32>,
    #[serde(rename = "groups")]
    pub groups: Option<Vec<Group>>,
    #[serde(rename = "members")]
    pub members: Option<Vec<Artist>>,
}

#[derive(Debug, serde::Deserialize)]
pub struct Arranger {
    #[serde(rename = "id")]
    pub id: i32,
    #[serde(rename = "names")]
    pub names: Vec<String>,
    #[serde(rename = "line_up_id")]
    pub line_up_id: Option<i32>,
    #[serde(rename = "groups")]
    pub groups: Option<Vec<Group>>,
    #[serde(rename = "members")]
    pub members: Option<Vec<Artist>>,
}

#[derive(Debug, serde::Deserialize)]
pub struct AnnMetadata {
    #[serde(rename = "annId")]
    pub ann_id: i32,
    #[serde(rename = "annSongId")]
    pub ann_song_id: i32,
    #[serde(rename = "animeENName")]
    pub anime_en_name: String,
    #[serde(rename = "animeJPName")]
    pub anime_jp_name: String,
    #[serde(rename = "animeAltName")]
    pub anime_alt_name: Option<String>,
    #[serde(rename = "animeVintage")]
    pub anime_vintage: String,
    #[serde(rename = "animeType")]
    pub anime_type: String,
    #[serde(rename = "songType")]
    pub song_type: String,
    #[serde(rename = "songName")]
    pub song_name: String,
    #[serde(rename = "songArtist")]
    pub song_artist: String,
    #[serde(rename = "songDifficulty")]
    pub song_difficulty: f32,
    #[serde(rename = "songCategory")]
    pub song_category: String,
    #[serde(rename = "HQ")]
    pub hq: Option<String>,
    #[serde(rename = "MQ")]
    pub mq: Option<String>,
    #[serde(rename = "audio")]
    pub audio: String,
    #[serde(rename = "artists")]
    pub artists: Vec<Artist>,
    #[serde(rename = "composers")]
    pub composers: Vec<Composer>,
    #[serde(rename = "arrangers")]
    pub arrangers: Vec<Arranger>,
}
