use serde::{Deserialize, Serialize};

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
