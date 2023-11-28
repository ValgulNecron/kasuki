use crate::sqls::sqlite::pool::get_sqlite_pool;

pub async fn get_guild_langage(guild_id: String) -> String {
    let database_url = "./data.db";
    let pool = get_sqlite_pool(database_url).await;
    let row: (Option<String>, Option<String>) =
        sqlx::query_as("SELECT lang, guild FROM guild_lang WHERE guild = ?")
            .bind(guild_id)
            .fetch_one(&pool)
            .await
            .unwrap_or((None, None));
    let (lang, _): (Option<String>, Option<String>) = row;

    lang.unwrap_or("en".to_string())
}
