use regex::Regex;

pub fn convert_to_discord_markdown(value: String) -> String {
    let mut result;
    result = convert_italic(value);
    result = convert_html_entity_to_real_char(result);
    result = convert_link_to_discord_markdown(result);
    result = add_anti_slash(result);
    result = convert_html_line_break_to_line_break(result);
    result = convert_bold(result);
    result = convert_spoiler(result);
    result = convert_strikethrough(result);

    result
}

pub fn convert_italic(value: String) -> String {
    let result = value
        .replace("<i>", "_")
        .replace("</i>", "_")
        .replace("<em>", "_")
        .replace("</em>", "_");
    result
}

pub fn convert_html_entity_to_real_char(value: String) -> String {
    let result = value.replace("&mdash;", "—");
    result
}

pub fn convert_link_to_discord_markdown(value: String) -> String {
    let re = Regex::new(r#"<a\s+href="([^"]+)">([^<]+)</a>"#).unwrap();
    let markdown = re.replace_all(&*value, "[$2]($1)");
    markdown.to_string()
}

pub fn add_anti_slash(value: String) -> String {
    let result = value.replace("`", "\\`");
    return result;
}

pub fn convert_html_line_break_to_line_break(value: String) -> String {
    let result = value.replace("<br>", "\n");
    result
}

pub fn convert_spoiler(value: String) -> String {
    let result = value.replace("~!", "||").replace("!~", "||");
    result
}

pub fn convert_bold(value: String) -> String {
    let result = value
        .replace("__", "**")
        .replace("<strong>", "**")
        .replace("</strong>", "**")
        .replace("<b>", "**")
        .replace("</b>", "**");
    result
}

pub fn convert_strikethrough(value: String) -> String {
    let result = value
        .replace("~~", "__")
        .replace("<del>", "__")
        .replace("</del>", "__")
        .replace("<strike>", "__")
        .replace("</strike>", "__");
    result
}
