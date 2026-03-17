use anyhow::{anyhow, Result};
use bytes::Bytes;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use reqwest::{multipart, Client};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::{error, trace};

use crate::image_saver::storage::ImageStore;

/// Normalize an API base URL by appending the given endpoint path.
///
/// Handles trailing `v1/`, `v1`, or no version prefix.
pub fn normalize_api_url(base_url: &str, endpoint: &str) -> String {
	if base_url.ends_with("v1/") {
		format!("{}{}", base_url, endpoint)
	} else if base_url.ends_with("v1") {
		format!("{}/{}", base_url, endpoint)
	} else {
		format!("{}/v1/{}", base_url, endpoint)
	}
}

/// Construct the URL for the chat/completions endpoint.
pub fn question_api_url(api_url: &str) -> String {
	normalize_api_url(api_url, "chat/completions")
}

/// Send a text prompt to an OpenAI-compatible chat API and return the response.
pub async fn question(
	text: &str, api_key: &str, api_base_url: &str, model: &str, http_client: &Client,
) -> Result<String> {
	let api_url = normalize_api_url(api_base_url, "chat/completions");

	let mut headers = HeaderMap::new();
	headers.insert(
		AUTHORIZATION,
		HeaderValue::from_str(&format!("Bearer {}", api_key))?,
	);
	headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

	let data = json!({
		"model": model,
		"messages": [
			{"role": "system", "content": "You are a helpful assistant."},
			{"role": "user", "content": text}
		]
	});

	trace!("{:?}", data);

	let res: Value = http_client
		.post(api_url)
		.headers(headers)
		.json(&data)
		.send()
		.await?
		.json()
		.await?;

	trace!("{:?}", res);

	let content = res["choices"][0]["message"]["content"].to_string();
	let content = content[1..content.len() - 1].to_string();

	Ok(content.replace("\\n", " \n "))
}

/// Translate text to a target language using an OpenAI-compatible chat API.
pub async fn translation(
	lang: &str, text: &str, api_key: &str, api_url: &str, model: &str, http_client: &Client,
) -> Result<String> {
	let prompt_gpt = format!(
		"\
            i will give you a text and a ISO-639-1 code and you will translate it in the corresponding language
            iso code: {}
            text:
            {}
            ",
		lang, text
	);

	let mut headers = HeaderMap::new();
	headers.insert(
		AUTHORIZATION,
		HeaderValue::from_str(&format!("Bearer {}", api_key))?,
	);
	headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

	let data = json!({
		"model": model,
		"messages": [
			{"role": "system", "content": "You are a expert in translating and only do that."},
			{"role": "user", "content": prompt_gpt}
		]
	});

	let res: Value = http_client
		.post(api_url)
		.headers(headers)
		.json(&data)
		.send()
		.await?
		.json()
		.await?;

	let content = res["choices"][0]["message"]["content"].to_string();
	let no_quote = content.replace('"', "");

	Ok(no_quote.replace("\\n", " \n "))
}

/// Build the JSON payload for an image generation request.
pub fn build_image_payload(
	prompt: &str, n: i64, model: &str, quality: Option<&str>, style: Option<&str>, size: &str,
) -> Value {
	let mut data = json!({
		"prompt": prompt,
		"n": n,
		"size": size,
		"model": model,
		"response_format": "url"
	});

	if let Some(quality) = quality {
		data["quality"] = json!(quality);
	}
	if let Some(style) = style {
		data["style"] = json!(style);
	}

	data
}

/// Download images from an API response, saving them to the image store.
///
/// Returns the raw image bytes for each image.
pub async fn download_images_from_response(
	json: Value, user_id: &str, guild_id: &str, image_store: &Arc<dyn ImageStore>,
	client: &Client,
) -> Result<Vec<Bytes>> {
	let mut bytes = Vec::new();

	let root: ImageRoot = match serde_json::from_value(json.clone()) {
		Ok(root) => root,
		Err(e) => {
			error!("Failed to deserialize response into ImageRoot: {}", e);
			error!("Raw response body: {}", json);
			if let Ok(err_root) = serde_json::from_value::<ImageErrorRoot>(json) {
				return Err(anyhow!(
					"Image generation API error: {}",
					err_root.error.message
				));
			}
			return Err(anyhow!("Failed to parse image generation response: {}", e));
		},
	};

	let urls: Vec<String> = root
		.data
		.iter()
		.filter_map(|data| data.url.clone())
		.collect();

	trace!("{:?}", urls);

	for url in &urls {
		let res = client.get(url).send().await?;
		let body = res.bytes().await?;

		let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
		let storage_key = format!("ai_images/ai_{}_{}_{}.png", user_id, guild_id, timestamp);

		if let Err(e) = image_store.save(&storage_key, &body).await {
			error!("Error saving image: {}", e);
		}

		bytes.push(body);
	}

	Ok(bytes)
}

/// Transcribe audio/video content using an OpenAI-compatible transcription API.
///
/// Returns the transcribed text.
pub async fn transcribe(
	buffer: Vec<u8>, content_type: &str, filename: &str, prompt: &str, lang: &str, token: &str,
	model: &str, api_base_url: &str, http_client: &Client,
) -> Result<String> {
	let url = normalize_api_url(api_base_url, "audio/transcriptions/");

	let mut headers = HeaderMap::new();
	headers.insert(
		AUTHORIZATION,
		HeaderValue::from_str(&format!("Bearer {}", token))?,
	);

	let part = multipart::Part::bytes(buffer)
		.file_name(filename.to_string())
		.mime_str(content_type)?;
	let form = multipart::Form::new()
		.part("file", part)
		.text("model", model.to_string())
		.text("prompt", prompt.to_string())
		.text("language", lang.to_string())
		.text("response_format", "json");

	let response = http_client
		.post(url)
		.headers(headers)
		.multipart(form)
		.send()
		.await?;

	let res: Value = response.json().await?;
	let text = res["text"].as_str().unwrap_or("").to_string();

	Ok(text)
}

/// Translate audio/video content using an OpenAI-compatible audio translation API.
///
/// This uses the audio/translations endpoint (different from text `translation`).
pub async fn translate_audio(
	buffer: Vec<u8>, content_type: &str, filename: &str, lang: &str, token: &str, model: &str,
	api_base_url: &str, http_client: &Client,
) -> Result<String> {
	let url = normalize_api_url(api_base_url, "audio/translations/");

	let mut headers = HeaderMap::new();
	headers.insert(
		AUTHORIZATION,
		HeaderValue::from_str(&format!("Bearer {}", token))?,
	);

	let part = multipart::Part::bytes(buffer)
		.file_name(filename.to_string())
		.mime_str(content_type)?;
	let form = multipart::Form::new()
		.part("file", part)
		.text("model", model.to_string())
		.text("language", lang.to_string())
		.text("response_format", "json");

	let response = http_client
		.post(url)
		.headers(headers)
		.multipart(form)
		.send()
		.await?;

	let res: Value = response.json().await?;
	let text = res["text"].as_str().unwrap_or("").to_string();

	Ok(text)
}

#[derive(Debug, Deserialize)]
struct ImageRoot {
	#[serde(rename = "data")]
	data: Vec<ImageData>,
}

#[derive(Debug, Deserialize)]
struct ImageData {
	url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AiError {
	pub message: String,
	#[serde(rename = "type")]
	pub error_type: String,
	pub param: Option<String>,
	pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ImageErrorRoot {
	pub error: AiError,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_normalize_api_url_with_v1_slash() {
		let result = normalize_api_url("https://api.openai.com/v1/", "chat/completions");
		assert_eq!(result, "https://api.openai.com/v1/chat/completions");
	}

	#[test]
	fn test_normalize_api_url_with_v1() {
		let result = normalize_api_url("https://api.openai.com/v1", "chat/completions");
		assert_eq!(result, "https://api.openai.com/v1/chat/completions");
	}

	#[test]
	fn test_normalize_api_url_without_v1() {
		let result = normalize_api_url("https://api.openai.com", "chat/completions");
		assert_eq!(result, "https://api.openai.com/v1/chat/completions");
	}

	#[test]
	fn test_question_api_url() {
		assert_eq!(
			question_api_url("https://api.openai.com/v1/"),
			"https://api.openai.com/v1/chat/completions"
		);
		assert_eq!(
			question_api_url("https://api.openai.com/v1"),
			"https://api.openai.com/v1/chat/completions"
		);
		assert_eq!(
			question_api_url("https://api.openai.com"),
			"https://api.openai.com/v1/chat/completions"
		);
	}

	#[test]
	fn test_normalize_image_url() {
		assert_eq!(
			normalize_api_url("https://api.openai.com/v1/", "images/generations"),
			"https://api.openai.com/v1/images/generations"
		);
		assert_eq!(
			normalize_api_url("https://api.openai.com/v1", "images/generations"),
			"https://api.openai.com/v1/images/generations"
		);
		assert_eq!(
			normalize_api_url("https://api.openai.com", "images/generations"),
			"https://api.openai.com/v1/images/generations"
		);
	}

	#[test]
	fn test_normalize_transcription_url() {
		assert_eq!(
			normalize_api_url("https://api.openai.com/v1/", "audio/transcriptions/"),
			"https://api.openai.com/v1/audio/transcriptions/"
		);
	}

	#[test]
	fn test_build_image_payload_all_options() {
		let payload =
			build_image_payload("a cat", 2, "dall-e-3", Some("hd"), Some("vivid"), "1024x1024");
		assert_eq!(payload["prompt"], "a cat");
		assert_eq!(payload["n"], 2);
		assert_eq!(payload["model"], "dall-e-3");
		assert_eq!(payload["quality"], "hd");
		assert_eq!(payload["style"], "vivid");
		assert_eq!(payload["size"], "1024x1024");
		assert_eq!(payload["response_format"], "url");
	}

	#[test]
	fn test_build_image_payload_no_optional() {
		let payload = build_image_payload("a dog", 1, "dall-e-2", None, None, "512x512");
		assert_eq!(payload["prompt"], "a dog");
		assert_eq!(payload["n"], 1);
		assert_eq!(payload["model"], "dall-e-2");
		assert!(payload.get("quality").map_or(true, |v| v.is_null()));
		assert!(payload.get("style").map_or(true, |v| v.is_null()));
		assert_eq!(payload["size"], "512x512");
	}

	#[test]
	fn test_build_image_payload_quality_only() {
		let payload = build_image_payload("test", 1, "dall-e-3", Some("standard"), None, "1024x1024");
		assert_eq!(payload["quality"], "standard");
		assert!(payload.get("style").map_or(true, |v| v.is_null()));
	}

	#[test]
	fn test_build_image_payload_style_only() {
		let payload = build_image_payload("test", 1, "dall-e-3", None, Some("natural"), "1024x1024");
		assert!(payload.get("quality").map_or(true, |v| v.is_null()));
		assert_eq!(payload["style"], "natural");
	}
}
