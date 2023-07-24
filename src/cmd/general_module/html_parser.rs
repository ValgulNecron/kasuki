use regex::Regex;

pub fn convert_to_markdown(value: String) -> String {
    let mut result;
    result = convert_i_to_markdown(value);
    result = convert_mdash_to_dash(result);
    result = convert_a_href_to_markdown(result);
    result = add_anti_slash(result);
    result = convert_br_to_line_break(result);
    result = convert_b_to_markdown(result);
    result = convert_spoiler(result);

    return result;
}

pub fn convert_i_to_markdown(value: String) -> String {
    let result = value.replace("<i>", "_").replace("</i>", "_");
    result
}

pub fn convert_mdash_to_dash(value: String) -> String {
    let result = value.replace("&mdash;", "—");
    result
}

pub fn convert_a_href_to_markdown(value: String) -> String {
    let re = Regex::new(r#"<a\s+href="([^"]+)">([^<]+)</a>"#).unwrap();
    let markdown = re.replace_all(&*value, "[$2]($1)");
    markdown.to_string()
}

pub fn add_anti_slash(value: String) -> String {
    let result = value.replace("`", "\\`");
    return result;
}

pub fn convert_br_to_line_break(value: String) -> String {
    let result = value.replace("<br>", "\n");
    result
}

pub fn convert_b_to_markdown(value: String) -> String {
    let result = value.replace("<b>", "**").replace("</b>", "**");
    result
}

pub fn convert_spoiler(value: String) -> String {
    let result = value.replace("~!", "||").replace("!~", "||");
    result
}
