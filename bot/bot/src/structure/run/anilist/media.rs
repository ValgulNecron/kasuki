pub use shared::anilist::media::*;

use crate::command::embed_content::{EmbedContent, EmbedsContents};
use anyhow::{anyhow, Result};
use fluent_templates::Loader;
use sea_orm::{entity::*, query::*, DatabaseConnection};
use shared::database::anime_song::Column::AnilistId;
use shared::database::prelude::AnimeSong;
use shared::localization::USABLE_LOCALES;
use std::fmt::Write;
use std::sync::Arc;
use unic_langid::LanguageIdentifier;

pub async fn media_content<'a>(
	data: Media, lang_id: &LanguageIdentifier, db: Arc<DatabaseConnection>,
) -> Result<EmbedsContents<'a>> {
	let anime_song = AnimeSong::find()
		.filter(AnilistId.eq(data.id.to_string()))
		.all(&*db)
		.await?;

	let mut song_list = anime_song
		.into_iter()
		.map(|song| {
			let mut message = song.song_name;
			if !song.audio.is_empty() {
				write!(message, " | [mp3]({})", song.audio).unwrap();
			}
			if !song.hq.is_empty() {
				write!(message, " | [mp4]({})", song.hq).unwrap();
			} else if !song.mq.is_empty() {
				write!(message, " | [mp4]({})", song.mq).unwrap();
			}
			message.push('\n');
			message
		})
		.collect::<String>();

	if song_list.len() > 1024 {
		song_list.truncate(1024);
		while !song_list.ends_with('\n') {
			song_list.pop();
		}
	}

	let mut fields = Vec::new();

	if !song_list.is_empty() {
		fields.push((
			USABLE_LOCALES.lookup(&lang_id, "anilist_user_media-song"),
			song_list,
			false,
		));
	}

	let genres: Vec<&str> = data
		.genres
		.as_ref()
		.map(|g| g.iter().flatten().take(5).map(|s| s.as_str()).collect())
		.unwrap_or_default();

	if !genres.is_empty() {
		fields.push((
			USABLE_LOCALES.lookup(&lang_id, "anilist_user_media-genre"),
			genres.join(","),
			false,
		))
	}

	if let Some(staff) = &data.staff {
		if let Some(edges) = staff.edges.clone() {
			let staffs = get_staff(edges);

			if !staffs.is_empty() {
				fields.push((
					USABLE_LOCALES.lookup(&lang_id, "anilist_user_media-staffs"),
					staffs,
					false,
				));
			}
		}
	}

	if let Some(characters) = &data.characters {
		if let Some(edges) = characters.edges.clone() {
			let chars = get_characters(edges);

			if !chars.is_empty() {
				fields.push((
					USABLE_LOCALES.lookup(&lang_id, "anilist_user_media-characters"),
					chars,
					false,
				));
			}
		}
	}

	let tags: Vec<&str> = data
		.tags
		.as_ref()
		.map(|t| t.iter().flatten().take(5).map(|t| t.name.as_str()).collect())
		.unwrap_or_default();

	if !tags.is_empty() {
		fields.push((
			USABLE_LOCALES.lookup(&lang_id, "anilist_user_media-tag"),
			tags.join(", "),
			false,
		));
	}

	if let Some(format) = data.format {
		fields.push((
			USABLE_LOCALES.lookup(&lang_id, "anilist_user_media-format"),
			format.to_string(),
			true,
		))
	}

	if let Some(source) = data.source {
		fields.push((
			USABLE_LOCALES.lookup(&lang_id, "anilist_user_media-source"),
			source.to_string(),
			true,
		))
	}

	if let Some(start_date) = &data.start_date {
		let mut start_date_str = String::new();

		if let Some(day) = start_date.day {
			write!(start_date_str, "{}/", day).unwrap();
		}

		if let Some(month) = start_date.month {
			write!(start_date_str, "{}/", month).unwrap();
		}

		if let Some(year) = start_date.year {
			write!(start_date_str, "{}", year).unwrap();
		}

		if !start_date_str.is_empty() {
			fields.push((
				USABLE_LOCALES.lookup(&lang_id, "anilist_user_media-start_date"),
				start_date_str,
				true,
			));
		}
	}

	if let Some(end_date) = &data.end_date {
		let mut end_date_str = String::new();

		if let Some(day) = end_date.day {
			write!(end_date_str, "{}/", day).unwrap();
		}

		if let Some(month) = end_date.month {
			write!(end_date_str, "{}/", month).unwrap();
		}

		if let Some(year) = end_date.year {
			write!(end_date_str, "{}", year).unwrap();
		}

		if !end_date_str.is_empty() {
			fields.push((
				USABLE_LOCALES.lookup(&lang_id, "anilist_user_media-end_date"),
				end_date_str,
				true,
			));
		}
	}

	if let Some(favourites) = data.favourites {
		fields.push((
			USABLE_LOCALES.lookup(&lang_id, "anilist_user_media-fav"),
			favourites.to_string(),
			true,
		))
	}

	if let Some(duration) = data.duration {
		fields.push((
			USABLE_LOCALES.lookup(&lang_id, "anilist_user_media-duration"),
			format!(
				"{} {}",
				duration,
				USABLE_LOCALES.lookup(&lang_id, "anilist_user_media-minutes")
			),
			true,
		));
	}

	if let Some(chapters) = data.chapters {
		fields.push((
			USABLE_LOCALES.lookup(&lang_id, "anilist_user_media-chapter"),
			chapters.to_string(),
			true,
		));
	}

	let title = data.title.as_ref().ok_or_else(|| anyhow!("No title"))?;

	let mut embed_content = EmbedContent::new(embed_title(title))
		.url(get_media_url(&data))
		.fields(fields)
		.images_url(get_banner(&data));

	if let Some(image) = data.cover_image {
		if let Some(extra_large) = image.extra_large {
			embed_content = embed_content.images_url(extra_large);
		}
	}

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embed_contents)
}

#[cfg(test)]
mod tests {
	use super::*;

	fn make_title(english: Option<&str>, romaji: Option<&str>) -> MediaTitle {
		MediaTitle {
			english: english.map(|s| s.to_string()),
			romaji: romaji.map(|s| s.to_string()),
		}
	}

	fn make_media() -> Media {
		Media {
			id: 1,
			cover_image: None,
			title: None,
			source: None,
			site_url: None,
			genres: None,
			favourites: None,
			format: None,
			duration: None,
			staff: None,
			start_date: None,
			end_date: None,
			chapters: None,
			characters: None,
			tags: None,
		}
	}

	#[test]
	fn embed_title_both_present() {
		let title = make_title(Some("Attack on Titan"), Some("Shingeki no Kyojin"));
		assert_eq!(
			embed_title(&title),
			"Attack on Titan / Shingeki no Kyojin"
		);
	}

	#[test]
	fn embed_title_english_only() {
		let title = make_title(Some("Attack on Titan"), None);
		assert_eq!(embed_title(&title), "Attack on Titan");
	}

	#[test]
	fn embed_title_romaji_only() {
		let title = make_title(None, Some("Shingeki no Kyojin"));
		assert_eq!(embed_title(&title), "Shingeki no Kyojin");
	}

	#[test]
	fn embed_title_neither() {
		let title = make_title(None, None);
		assert_eq!(embed_title(&title), "");
	}

	#[test]
	fn embed_title_empty_strings() {
		let title = make_title(Some(""), Some(""));
		assert_eq!(embed_title(&title), "");
	}

	#[test]
	fn embed_title_english_empty_romaji_present() {
		let title = make_title(Some(""), Some("Shingeki no Kyojin"));
		assert_eq!(embed_title(&title), "Shingeki no Kyojin");
	}

	#[test]
	fn get_url_with_site_url() {
		let mut media = make_media();
		media.site_url = Some("https://anilist.co/anime/16498".to_string());
		assert_eq!(get_media_url(&media), "https://anilist.co/anime/16498");
	}

	#[test]
	fn get_url_without_site_url() {
		let media = make_media();
		assert_eq!(get_media_url(&media), "https://example.com");
	}

	#[test]
	fn get_banner_formats_correctly() {
		let mut media = make_media();
		media.id = 16498;
		assert_eq!(get_banner(&media), "https://img.anili.st/media/16498");
	}

	#[test]
	fn get_staff_formats_names_and_roles() {
		let staff = vec![
			Some(StaffEdge {
				role: Some("Director".to_string()),
				node: Some(Staff {
					name: Some(StaffName {
						user_preferred: Some("Tetsuro Araki".to_string()),
						full: Some("Tetsuro Araki Full".to_string()),
					}),
				}),
			}),
			Some(StaffEdge {
				role: Some("Music".to_string()),
				node: Some(Staff {
					name: Some(StaffName {
						user_preferred: None,
						full: Some("Hiroyuki Sawano".to_string()),
					}),
				}),
			}),
		];
		let result = get_staff(staff);
		assert!(result.contains("Tetsuro Araki: Director"));
		assert!(result.contains("Hiroyuki Sawano: Music"));
	}

	#[test]
	fn get_staff_caps_at_5() {
		let staff: Vec<Option<StaffEdge>> = (0..10)
			.map(|i| {
				Some(StaffEdge {
					role: Some(format!("Role {}", i)),
					node: Some(Staff {
						name: Some(StaffName {
							user_preferred: Some(format!("Staff {}", i)),
							full: None,
						}),
					}),
				})
			})
			.collect();
		let result = get_staff(staff);
		let lines: Vec<&str> = result.trim().lines().collect();
		assert_eq!(lines.len(), 5);
	}

	#[test]
	fn get_staff_skips_none_entries() {
		let staff = vec![
			None,
			Some(StaffEdge {
				role: Some("Director".to_string()),
				node: Some(Staff {
					name: Some(StaffName {
						user_preferred: Some("Name".to_string()),
						full: None,
					}),
				}),
			}),
			None,
		];
		let result = get_staff(staff);
		assert_eq!(result.trim().lines().count(), 1);
	}

	#[test]
	fn get_characters_formats_names() {
		let chars = vec![
			Some(CharacterEdge {
				node: Some(Character {
					name: Some(CharacterName {
						user_preferred: Some("Eren Yeager".to_string()),
						full: None,
					}),
				}),
			}),
			Some(CharacterEdge {
				node: Some(Character {
					name: Some(CharacterName {
						user_preferred: None,
						full: Some("Mikasa Ackerman".to_string()),
					}),
				}),
			}),
		];
		let result = get_characters(chars);
		assert!(result.contains("Eren Yeager"));
		assert!(result.contains("Mikasa Ackerman"));
	}

	#[test]
	fn get_characters_caps_at_5() {
		let chars: Vec<Option<CharacterEdge>> = (0..10)
			.map(|i| {
				Some(CharacterEdge {
					node: Some(Character {
						name: Some(CharacterName {
							user_preferred: Some(format!("Char {}", i)),
							full: None,
						}),
					}),
				})
			})
			.collect();
		let result = get_characters(chars);
		let lines: Vec<&str> = result.trim().lines().collect();
		assert_eq!(lines.len(), 5);
	}

	#[test]
	fn media_format_display() {
		assert_eq!(MediaFormat::Tv.to_string(), "TV");
		assert_eq!(MediaFormat::TvShort.to_string(), "TV Short");
		assert_eq!(MediaFormat::Movie.to_string(), "Movie");
		assert_eq!(MediaFormat::Novel.to_string(), "Novel");
		assert_eq!(MediaFormat::OneShot.to_string(), "One Shot");
	}

	#[test]
	fn media_source_display() {
		assert_eq!(MediaSource::Original.to_string(), "Original");
		assert_eq!(MediaSource::LightNovel.to_string(), "Light Novel");
		assert_eq!(MediaSource::VisualNovel.to_string(), "Visual Novel");
		assert_eq!(MediaSource::WebNovel.to_string(), "Web Novel");
	}

	#[test]
	fn media_type_display() {
		assert_eq!(MediaType::Anime.to_string(), "Anime");
		assert_eq!(MediaType::Manga.to_string(), "Manga");
	}
}
