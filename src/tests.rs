#[cfg(test)]
mod tests {
    use std::any::{Any, TypeId};

    use serde_json::json;
    use sqlx::{Pool, Sqlite};

    use crate::cmd::general_module::get_guild_langage::get_guild_langage;
    use crate::cmd::general_module::html_parser::{
        add_anti_slash, convert_bold, convert_html_entity_to_real_char,
        convert_html_line_break_to_line_break, convert_italic, convert_link_to_discord_markdown,
        convert_spoiler, convert_to_discord_markdown,
    };
    use crate::cmd::general_module::pool::get_pool;
    use crate::cmd::general_module::request::make_request_anilist;
    use crate::cmd::general_module::trim::trim;

    #[test]
    fn test_parser_mdash() {
        assert_eq!(
            convert_html_entity_to_real_char("&mdash; test &mdash;".to_string()),
            "— test —"
        );
    }

    #[test]
    fn test_parser_italic() {
        assert_eq!(convert_italic("<i>test</i>".to_string()), "_test_");
    }

    #[test]
    fn test_parser_href() {
        assert_eq!(
            convert_link_to_discord_markdown(
                "<a href=\"https://anilist.co/character/138101/Loid-Forger\">Loid Forger</a>"
                    .to_string()
            ),
            "[Loid Forger](https://anilist.co/character/138101/Loid-Forger)"
        );
    }

    #[test]
    fn test_parser_anti_slash() {
        assert_eq!(add_anti_slash("Brother`s".to_string()), "Brother\\`s");
    }

    #[test]
    fn test_parser_line_break() {
        assert_eq!(
            convert_html_line_break_to_line_break("<br> test".to_string()),
            "\n test"
        );
    }

    #[test]
    fn test_parser_bold() {
        assert_eq!(convert_bold("<b>test</b>".to_string()), "**test**");
    }

    #[test]
    fn test_parser_spoiler1() {
        assert_eq!(convert_spoiler("~!test!~".to_string()), "||test||");
    }

    #[test]
    fn test_parser_spoiler2() {
        assert_eq!(
            convert_spoiler("~!test!~\n ~!test!~ ~!test!~".to_string()),
            "||test||\n ||test|| ||test||"
        );
    }

    #[test]
    fn test_parser_complete() {
        assert_eq!(convert_to_discord_markdown("~!test!~\n ~!test!~ ~!test!~ <b>test</b> <br> test Brother`s <a href=\"https://anilist.co/character/138101/Loid-Forger\">Loid Forger</a> <i>test</i> &mdash; test &mdash;"
            .to_string()), "||test||\n ||test|| ||test|| **test** \n test Brother\\`s [Loid Forger](https://anilist.co/character/138101/Loid-Forger) _test_ — test —");
    }

    #[test]
    fn test_trim_less() {
        let desc = "In the serene forest, the rustling leaves and chirping birds created a peaceful melody. The sun gently kissed the earth, painting the sky with hues of orange and pink, as nature embraced its tranquil symphony.".to_string();
        let lenght_diff = 4096 - desc.len() as i32;
        let result_len;
        if lenght_diff <= 0 {
            result_len = trim(desc, lenght_diff).len()
        } else {
            result_len = 0;
        }
        assert!(result_len < 4096)
    }

    #[test]
    fn test_trim_more() {
        let desc = "In the serene forest, the rustling leaves and chirping birds created a peaceful melody. The sun gently kissed the earth, painting the sky with hues of orange and pink, as nature embraced its tranquil symphony. The fragrance of wildflowers filled the air, and a gentle breeze caressed the leaves, carrying the whispers of the ancient trees. As the day unfolded, the forest awakened with life. Squirrels darted through the branches, and deer gracefully danced in the clearings. The harmonious chorus of crickets and cicadas added to the symphony of nature, captivating all who listened. Beyond the forest's edge, a meandering river sparkled under the warm rays of the sun. Dragonflies flitted over the water, while fish swam gracefully beneath the surface. The river's gentle flow seemed to echo the rhythm of the forest, blending in perfect harmony. As the evening approached, the forest transformed into a magical realm. Fireflies emerged, their ethereal glow illuminating the darkening woods. The stars appeared one by one, painting the night sky with their celestial beauty.  In this enchanted world, time seemed to slow down, and worries faded away. The forest was a sanctuary of tranquility, a place where one could connect with the essence of life itself. It reminded all who wandered through its depths of the profound interconnectedness of all living beings. Under the moon's gentle gaze, the forest exuded an aura of mystery and wonder. Legends whispered in the wind, tales of ancient spirits and mystical creatures. Each rustle of leaves seemed to carry a secret, inviting the curious to explore the unknown. The forest's embrace was a balm for the soul, a source of solace and inspiration. It reminded humanity of its humble place in the grand tapestry of the universe, urging reverence for the natural world. As night turned to dawn, the forest prepared for a new day. The first rays of sunlight gently filtered through the canopy, casting a soft glow on the forest floor. Creatures big and small stirred from their slumber, greeting the dawn with anticipation. And so, the timeless dance of life continued, day after day, season after season. The serene forest remained an eternal witness to the ever-changing cycle of existence, a sanctuary of beauty and wisdom for all who sought its embrace. In the serene forest, the rustling leaves and chirping birds created a peaceful melody. The sun gently kissed the earth, painting the sky with hues of orange and pink, as nature embraced its tranquil symphony. The fragrance of wildflowers filled the air, and a gentle breeze caressed the leaves, carrying the whispers of the ancient trees. As the day unfolded, the forest awakened with life. Squirrels darted through the branches, and deer gracefully danced in the clearings. The harmonious chorus of crickets and cicadas added to the symphony of nature, captivating all who listened. Beyond the forest's edge, a meandering river sparkled under the warm rays of the sun. Dragonflies flitted over the water, while fish swam gracefully beneath the surface. The river's gentle flow seemed to echo the rhythm of the forest, blending in perfect harmony. As the evening approached, the forest transformed into a magical realm. Fireflies emerged, their ethereal glow illuminating the darkening woods. The stars appeared one by one, painting the night sky with their celestial beauty.  In this enchanted world, time seemed to slow down, and worries faded away. The forest was a sanctuary of tranquility, a place where one could connect with the essence of life itself. It reminded all who wandered through its depths of the profound interconnectedness of all living beings. Under the moon's gentle gaze, the forest exuded an aura of mystery and wonder. Legends whispered in the wind, tales of ancient spirits and mystical creatures. Each rustle of leaves seemed to carry a secret, inviting the curious to explore the unknown. The forest's embrace was a balm for the soul, a source of solace and inspiration. It reminded humanity of its humble place in the grand tapestry of the universe, urging reverence for the natural world. As night turned to dawn, the forest prepared for a new day. The first rays of sunlight gently filtered through the canopy, casting a soft glow on the forest floor. Creatures big and small stirred from their slumber, greeting the dawn with anticipation. And so, the timeless dance of life continued, day after day, season after season. The serene forest remained an eternal witness to the ever-changing cycle of existence, a sanctuary of beauty and wisdom for all who sought its embrace.".to_string();
        let lenght_diff = 4096 - desc.len() as i32;
        let result_len;
        if lenght_diff <= 0 {
            result_len = trim(desc, lenght_diff).len()
        } else {
            result_len = 0;
        }
        assert_eq!(result_len, 4096)
    }

    #[tokio::test]
    async fn test_guild_langage() {
        assert_eq!(
            get_guild_langage("1117152661620408531".to_string()).await,
            "En".to_string()
        );
    }

    #[tokio::test]
    async fn test_get_pool() {
        let pool = get_pool("./cache.db").await;
        assert_eq!(TypeId::of::<Pool<Sqlite>>(), pool.type_id());
    }

    #[tokio::test]
    async fn test_make_request() {
        let query: &str = "query ($search: Int = 5399974) {
            User(id: $search){
                    id
                }
            }";
        let json = json!({"query": query,});
        let resp = make_request_anilist(json, true).await;
        let good_resp = r#"{"data":{"User":{"id":5399974}}}"#;
        assert_eq!(resp, good_resp)
    }
}
