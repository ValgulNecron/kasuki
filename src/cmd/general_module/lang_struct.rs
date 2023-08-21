use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AnimeLocalisedText {
    pub desc_title: String,
    pub desc_part_1: String,
    pub desc_part_2: String,
    pub desc_part_3: String,
    pub desc_part_4: String,
    pub desc_part_5: String,
    pub desc_part_6: String,
    pub desc_part_7: String,
    pub fields_name_1: String,
    pub fields_name_2: String,
    pub error_slash_command: String,
    pub error_not_nsfw: String,
    pub error_anime_not_found: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MediaLocalisedText {
    pub full_name: String,
    pub user_pref: String,
    pub role: String,
    pub format: String,
    pub source: String,
    pub start_date: String,
    pub end_date: String,
    pub fields_name_1: String,
    pub fields_name_2: String,
    pub desc_title: String,
    pub error_slash_command: String,
    pub error_no_media: String,
    pub error_not_nsfw: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LevelLocalisedText {
    pub level: String,
    pub xp: String,
    pub progression_1: String,
    pub progression_2: String,
    pub error_slash_command: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RandomLocalisedText {
    pub error_title: String,
    pub error_message: String,
    pub genre: String,
    pub tag: String,
    pub format: String,
    pub desc: String,
    pub error_slash_command: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RegisterLocalisedText {
    pub part_1: String,
    pub part_2: String,
    pub part_3: String,
    pub error_slash_command: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StaffLocalisedText {
    pub desc_title: String,
    pub date_of_birth: String,
    pub date_of_death: String,
    pub hometown: String,
    pub primary_language: String,
    pub primary_occupation: String,
    pub media: String,
    pub va: String,
    pub error_slash_command: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserLocalisedText {
    pub manga_title: String,
    pub manga_count: String,
    pub manga_completed: String,
    pub manga_chapter_read: String,
    pub manga_mean_score: String,
    pub manga_standard_deviation: String,
    pub manga_pref_tag: String,
    pub manga_pref_genre: String,
    pub anime_title: String,
    pub anime_count: String,
    pub anime_completed: String,
    pub anime_time_watch: String,
    pub anime_mean_score: String,
    pub anime_standard_deviation: String,
    pub anime_pref_tag: String,
    pub anime_pref_genre: String,
    pub week: String,
    pub day: String,
    pub hour: String,
    pub minute: String,
    pub error_slash_command: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CompareLocalisedText {
    pub more_anime: String,
    pub connector_user_same_anime: String,
    pub same_anime: String,
    pub time_anime_watch: String,
    pub connector_user_same_time: String,
    pub same_time: String,
    pub more_manga: String,
    pub connector_user_same_manga: String,
    pub same_manga: String,
    pub more_chapter: String,
    pub connector_user_same_chapter: String,
    pub same_chapter: String,
    pub genre_same_connector_anime: String,
    pub genre_same_prefer_anime: String,
    pub diff_pref_genre_1_anime: String,
    pub diff_pref_genre_while_anime: String,
    pub diff_pref_genre_2_anime: String,
    pub same_tag_connector_anime: String,
    pub same_tag_prefer_anime: String,
    pub diff_pref_tag_1_anime: String,
    pub diff_pref_tag_while_anime: String,
    pub diff_pref_tag_2_anime: String,
    pub diff_pref_tag_anime: String,
    pub genre_same_connector_manga: String,
    pub genre_same_prefer_manga: String,
    pub diff_pref_genre_1_manga: String,
    pub diff_pref_genre_while_manga: String,
    pub diff_pref_genre_2_manga: String,
    pub diff_pref_genre_manga: String,
    pub same_tag_connector_manga: String,
    pub same_tag_prefer_manga: String,
    pub diff_pref_tag_1_manga: String,
    pub diff_pref_tag_while_manga: String,
    pub diff_pref_tag_2_manga: String,
    pub title: String,
    pub sub_title_anime: String,
    pub watch_time: String,
    pub pref_genre_anime: String,
    pub pref_tag_anime: String,
    pub sub_title_manga: String,
    pub chapter_read: String,
    pub pref_genre_manga: String,
    pub pref_tag_manga: String,
    pub error_slash_command: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StudioLocalisedText {
    pub anime_or_manga: String,
    pub error_slash_command: String,
    pub favorite: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AddActivityLocalisedText {
    pub error_no_media: String,
    pub title1: String,
    pub title2: String,
    pub already_added: String,
    pub adding: String,
    pub error_slash_command: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SendActivityLocalisedText {
    pub title: String,
    pub ep: String,
    pub of: String,
    pub end: String,
}
