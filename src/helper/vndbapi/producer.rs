use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub async fn get_producer(
    value: String,
) -> Result<ProducerRoot, crate::helper::error_management::error_enum::AppError> {
    let value = value.to_lowercase();
    let value = value.trim();
    let start_with_v = value.starts_with('v');
    let is_number = value.chars().skip(1).all(|c| c.is_numeric());
    let json = if start_with_v && is_number {
        (r#"{
    		"filters": ["id", "=",""#
            .to_owned()
            + value
            + r#""],
    		"fields": "id, name, original, aliases,lang,type,description"
		}"#)
        .to_string()
    } else {
        (r#"{
    		"filters": ["search", "=",""#
            .to_owned()
            + value
            + r#""],
    		"fields": "id, name, original, aliases,lang,type,description"
		}"#)
        .to_string()
    };
    let path = "/producer".to_string();
    let response =
        crate::helper::vndbapi::common::do_request_cached_with_json(path.clone(), json.to_string())
            .await?;
    let response: ProducerRoot = serde_json::from_str(&response).map_err(|e| {
        crate::helper::error_management::error_enum::AppError {
            message: format!("Error while parsing response: '{}'", e),
            error_type: crate::helper::error_management::error_enum::ErrorType::WebRequest,
            error_response_type:
                crate::helper::error_management::error_enum::ErrorResponseType::Unknown,
        }
    })?;
    Ok(response)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Producer {
    #[serde(rename = "type")]
    pub results_type: Option<Type>,

    pub lang: Option<String>,

    pub name: String,

    pub description: Option<String>,

    pub aliases: Option<Vec<String>>,

    pub id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProducerRoot {
    pub more: Option<bool>,

    pub results: Vec<Producer>,
}

#[derive(Debug, Clone)]
pub enum Type {
    Company,
    Individual,
    AmateurGroup,
    Unknown,
}

impl Serialize for Type {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = match self {
            Self::Company => "co",
            Self::Individual => "in",
            Self::AmateurGroup => "ng",
            Self::Unknown => "unknown", // Placeholder for unknown
        };
        value.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Type {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "co" => Ok(Self::Company),
            "in" => Ok(Self::Individual),
            "ng" => Ok(Self::AmateurGroup),
            _ => Err(serde::de::Error::custom("Invalid producer type")),
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Company => "Company",
            Self::Individual => "Individual",
            Self::AmateurGroup => "Amateur Group",
            Self::Unknown => "Unknown",
        };
        write!(f, "{}", value)
    }
}
