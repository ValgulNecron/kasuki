use std::error::Error;
use std::fmt::Display;
use std::sync::Arc;

use moka::future::Cache;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tokio::sync::RwLock;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Image {
    pub violence: f64,
    pub url: String,
    pub sexual: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Staff {
    pub id: String,
    pub name: String,
    pub role: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Titles {
    pub title: String,
    pub lang: String,
    pub official: bool,
    pub main: bool,
    pub latin: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tags {
    pub name: String,
    pub id: String,
    pub rating: f64,
    pub spoiler: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Developers {
    pub name: String,
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VN {
    pub olang: String,
    pub va: Vec<Va>,
    pub released: Option<String>,
    pub id: String,
    pub image: Option<Image>,
    pub staff: Vec<Staff>,
    pub rating: Option<f64>,
    pub length_minutes: Option<f64>, // Kept as Option since it might be null
    pub platforms: Vec<String>,
    pub title: String,
    pub average: Option<f64>,
    pub titles: Vec<Titles>,
    pub votecount: f64,
    pub languages: Vec<String>,
    pub aliases: Vec<String>,
    pub tags: Vec<Tags>,
    pub description: Option<String>,
    pub devstatus: DevStatus,
    pub developers: Vec<Developers>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VNRoot {
    pub results: Vec<VN>,
    pub more: bool,
}
pub async fn get_vn(
    value: String,
    vndb_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<VNRoot, Box<dyn Error>> {
    let value = value.to_lowercase();
    let value = value.trim();
    let start_with_v = value.starts_with('v');
    let is_number = value.chars().skip(1).all(|c| c.is_numeric());
    let json = if start_with_v && is_number {
        (r#"{
    		"filters": ["id", "=",""#.to_owned() + value + r#""],
    		"fields": "id,title,alttitle,titles.lang,titles.title,titles.latin,titles.official,titles.main, aliases,olang,devstatus,released,languages,platforms,image.url,image.sexual,image.violence,length_minutes,description,average,rating,votecount,tags.rating,tags.spoiler,tags.name,developers.name,staff.name,staff.role,va.character.name"
		}"#).to_string()
    } else {
        (r#"{
    		"filters": ["search", "=",""#.to_owned() + value + r#""],
    		"fields": "id,title,alttitle,titles.lang,titles.title,titles.latin,titles.official,titles.main, aliases,olang,devstatus,released,languages,platforms,image.url,image.sexual,image.violence,length_minutes,description,average,rating,votecount,tags.rating,tags.spoiler,tags.name,developers.name,staff.name,staff.role,va.character.name"
		}"#).to_string()
    };
    let path = "/vn".to_string();
    let response = crate::helper::vndbapi::common::do_request_cached_with_json(
        path.clone(),
        json.to_string(),
        vndb_cache,
    )
    .await?;
    let response: VNRoot = serde_json::from_str(&response)?;
    Ok(response)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Character {
    pub id: String,

    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Va {
    pub character: Character,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DevStatus {
    Finished,
    Development,
    Cancelled,
    Unknown,
}

impl Serialize for DevStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = match self {
            Self::Finished => 0,
            Self::Development => 1,
            Self::Cancelled => 2,
            Self::Unknown => 99, // Assuming 99 as a placeholder for unknown
        };
        value.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for DevStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = i8::deserialize(deserializer)?;
        match value {
            0 => Ok(Self::Finished),
            1 => Ok(Self::Development),
            2 => Ok(Self::Cancelled),
            _ => Ok(Self::Unknown),
        }
    }
}

impl Display for DevStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Finished => "Finished",
            Self::Development => "Development",
            Self::Cancelled => "Cancelled",
            Self::Unknown => "Unknown",
        };
        write!(f, "{}", value)
    }
}
