use crate::sqls::sqlite::data::get_data_guild_lang;

pub async fn get_guild_langage(guild_id: String) -> String {
    if guild_id == String::from("0") {
        return String::from("en");
    };

    let (lang, _): (Option<String>, Option<String>) =
        get_data_guild_lang(guild_id).await.unwrap_or((None, None));

    lang.unwrap_or("en".to_string())
}
