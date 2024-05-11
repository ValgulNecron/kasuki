use regex::Regex;

/// Converts the given string into Discord-flavored markdown
///
/// This function applies a series of transformations to a given string to
/// represent it in a format compatible with Discord's flavor of markdown. It will
/// convert italicized text, translate HTML entities into their actual characters,
/// convert hyperlink formats, add an anti-slash (backslash) where necessary,
/// convert HTML style line breaks to natural line breaks, convert bold text,
/// wrap spoilers in appropriate tags, and format strikethrough text.
///
/// # Arguments
///
/// - `value` - A `String` that represents the text to be converted to Discord-flavored markdown.
///
/// # Returns
///
/// A `String` that represents the text in Discord-flavored markdown.
pub fn convert_anilist_flavored_to_discord_flavored_markdown(value: String) -> String {
    let mut result = value;
    result = convert_italic(result);
    result = convert_bold(result);
    result = convert_strikethrough(result);
    result = remove_p_align(result);
    result = convert_blockquote(result);
    result = convert_link_to_discord_markdown(result);
    result = remove_image(result);
    result = convert_h_header(result);
    result = remove_horizontal_line(result);
    result = convert_list(result);
    result = remove_code_block(result);
    result = convert_spoiler(result);
    result = convert_html_entity_to_real_char(result);
    result = add_anti_slash(result);
    result = convert_html_line_break_to_line_break(result);

    result
}

/// Converts the given string into Discord-flavored markdown
///
/// This function applies a series of transformations to a given string to
/// represent it in a format compatible with Discord's flavor of markdown. It will
/// convert italicized text, translate HTML entities into their actual characters,
/// convert hyperlink formats, add an anti-slash (backslash) where necessary,
/// convert HTML style line breaks to natural line breaks, convert bold text,
/// wrap spoilers in appropriate tags, and format strikethrough text.
///
/// # Arguments
///
/// - `value` - A `String` that represents the text to be converted to Discord-flavored markdown.
///
/// # Returns
///
/// A `String` that represents the text in Discord-flavored markdown.
pub fn convert_steam_to_discord_flavored_markdown(value: String) -> String {
    let mut result = value;
    result = add_anti_slash(result);
    result = convert_html_entity_to_real_char(result);
    result = convert_link_to_discord_markdown(result);
    result = convert_html_line_break_to_line_break(result);
    result = convert_bold(result);
    result = convert_strikethrough(result);
    result = convert_blockquote(result);
    result = convert_h_header(result);
    result = convert_italic(result);

    result
}

/// Converts the HTML tags '<i>' and '<em>' (including their ending tags) in a given string to underscore, '_'
///
/// # Arguments
///
/// * `value` - A string that may contain '<i>', '</i>', '<em>', or '</em>' tags which need conversion.
///
/// # Returns
///
/// * An owned String where any '<i>', '</i>', '<em>', or '</em>' tags have been replaced with '_'.
///
/// # Examples
///
/// ```
/// let str = "<i>Hello</i> <em>World</em>";
/// let result = convert_italic(str);
/// assert_eq!(result, "_Hello_ _World_");
/// ```
pub fn convert_italic(value: String) -> String {
    value
        .replace("<i>", "_")
        .replace("</i>", "_")
        .replace("<em>", "_")
        .replace("</em>", "_")
}

/// Converts HTML entities in a given string to their corresponding characters.
///
/// This function takes a string as input and replaces all occurrences of certain HTML entities
/// with their corresponding characters. The HTML entities that are replaced include "&mdash;",
/// "&amp;", "&lt;", "&gt;", "&quot;", and "&apos;".
///
/// # Arguments
///
/// * `value` - A string that may contain HTML entities which need conversion.
///
/// # Returns
///
/// * An owned String where any HTML entities have been replaced with their corresponding characters.
///
/// # Examples
///
/// ```
/// let input = "Hello &mdash; World".to_string();
/// let output = convert_html_entity_to_real_char(input);
/// assert_eq!(output, "Hello — World");
/// ```
pub fn convert_html_entity_to_real_char(value: String) -> String {
    value
        .replace("&mdash;", "—")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

/// Convert HTML anchor tags in a string to Discord-flavored Markdown link.
///
/// This function takes a `String` value as input. It uses the `regex` crate to
/// construct a regular expression that matches HTML anchor links.
/// It replaces every HTML link in the input string with its equivalent in the
/// Discord-flavored Markdown syntax, which is `[link_text](url)`.
///
/// # Arguments
///
/// * `value` - A `String` that may contain HTML anchor links.
///
/// # Returns
///
/// A `String` which is the input with all HTML anchor links replaced by
/// Discord-flavored Markdown links. If no HTML anchor links are found, the
/// original string is returned.
pub fn convert_link_to_discord_markdown(value: String) -> String {
    let re = Regex::new(r#"<a\s+href="([^"]+)">([^<]+)</a>"#).unwrap();
    re.replace_all(value.as_str(), "[$2]($1)").to_string()
}

/// This function replaces a single backquote (`) within a given string with a backslash (\) followed by a backquote (`).
///
/// # Arguments
///
/// * `value` - A `String` that would be processed. It should contain backquote(s) (`) if you expect any change in the string.
///
/// # Returns
///
/// A `String` that is the result of replacing all the occurrences of single backquote (`) with a backslash followed by a backquote (`\``).
///
/// # Examples
///
/// ```
/// let input = String::from("Hello`World");
/// let result = add_anti_slash(input);
/// assert_eq!(result, String::from("Hello\\`World"));
/// ```
///
///
pub fn add_anti_slash(value: String) -> String {
    value.replace('`', "\\`")
}

/// # Convert HTML line breaks to regular line breaks
///
/// This function takes a `String` as an argument, where it
/// might contain HTML line breaks (`<br>`), and returns a new `String`
/// where those HTML line breaks have been converted into regular line
/// break characters (`\n`).
///
/// # Arguments
///
/// * `value` - The `String` to be converted
///
/// #  Returns
///
/// A `String` where HTML line breaks (`<br>`) have been replaced with regular line breaks (`\n`)
///
/// # Example
///
/// ```
/// let my_string = String::from("Hello<br>World");
/// let new_string = convert_html_line_break_to_line_break(my_string);
/// assert_eq!(new_string, "Hello\nWorld");
/// ```
///
pub fn convert_html_line_break_to_line_break(value: String) -> String {
    value.replace("<br>", "\n")
}

/// This function converts the placeholders of a spoiler tag in a string.
///
/// The original format of a spoiler tag is "~![spoiler_content]!~".
/// The converted format will be "||[spoiler_content]||".
///
/// Example:
/// ```
/// let original_string = "~!This is a spoiler!~";
/// let converted_string = convert_spoiler(original_string.to_string());
/// assert_eq!(converted_string, "||This is a spoiler||");
/// ```
///
/// # Parameters
///
/// - `value`: The original string that contains spoiler tags.
///
/// # Returns
///
/// A new String where the spoiler tags have been transformed.
pub fn convert_spoiler(value: String) -> String {
    value.replace("~!", "||").replace("!~", "||")
}

/// Converts the provided string's bold style from various formats (__, <strong>, <b>)
/// to the Markdown representation (**).
///
/// This function does not handle the scenario where the string contains unbalanced tags.
/// It simply replaces each occurrence of a tag or double underscore with double asterisks.
///
/// # Arguments
///
/// * `value` - A string that may contain text in a variety of bold formats.
///
/// # Returns
///
/// A string where all bold styles have been converted to the Markdown format.
///
/// # Examples
///
/// ```rust
/// let example = String::from("<b>Hello</b>, <strong>World!</strong>");
/// let result = convert_bold(example);
/// assert_eq!(result, "**Hello**, **World!**");
/// ```
pub fn convert_bold(value: String) -> String {
    value
        .replace("<strong>", "**")
        .replace("</strong>", "**")
        .replace("<b>", "**")
        .replace("</b>", "**")
}

/// Converts strikethrough markdown to underline markdown.
///
/// This function takes a String as input, checks for strikethrough Markdown syntax
/// (`~~`, `<del>`, and `<strike>`) and replaces them with underline syntax (`__`).
///
/// # Arguments
///
/// * `value` - A string slice that holds the content to be converted.
///
/// # Returns
///
/// A String with replaced 'Strikethrough' markdowns with 'Underline' markdown ones.
///
/// # Examples
///
/// ```Rust
/// let str = "This is a ~~test~~.";
/// let result = convert_strikethrough(str.to_string());
/// assert_eq!(result, "This is a __test__.");
/// ```
pub fn convert_strikethrough(value: String) -> String {
    value
        .replace("<del>", "~~")
        .replace("</del>", "~~")
        .replace("<strike>", "~~")
        .replace("</strike>", "~~")
}

/// Converts the HTML tags '<blockquote>' and '</blockquote>' in a given string to '>' and removes any extra '>' characters.
///
/// This function first replaces the '<blockquote>' and '</blockquote>' tags in the string with '>'.
/// It then uses a regular expression to find and remove any extra '>' characters.
///
/// # Arguments
///
/// * `value` - A string that may contain '<blockquote>' and '</blockquote>' tags which need conversion.
///
/// # Returns
///
/// * An owned String where any '<blockquote>' and '</blockquote>' tags have been replaced with '>' and any extra '>' characters have been removed.
pub fn convert_blockquote(value: String) -> String {
    let mut value = value
        .replace("<blockquote>", "> ")
        .replace("</blockquote>", "");

    let re = Regex::new(r#">+"#).unwrap();
    value = re.replace_all(value.as_str(), ">").to_string();

    value
}

/// Converts the HTML tags '<h1>', '<h2>', '<h3>', '<h4>', '<h5>', and '<h6>' (including their ending tags) in a given string to markdown headers.
///
/// This function first replaces the '<h1>', '<h2>', '<h3>', '<h4>', '<h5>', and '<h6>' tags in the string with their equivalent markdown headers.
/// It then uses a regular expression to find and replace any lines consisting only of '=' or '-' characters with '#' or '##' respectively.
///
/// # Arguments
///
/// * `value` - A string that may contain '<h1>', '<h2>', '<h3>', '<h4>', '<h5>', and '<h6>' tags which need conversion.
///
/// # Returns
///
/// * An owned String where any '<h1>', '<h2>', '<h3>', '<h4>', '<h5>', and '<h6>' tags have been replaced with their equivalent markdown headers and any lines consisting only of '=' or '-' characters have been replaced with '#' or '##' respectively.
pub fn convert_h_header(value: String) -> String {
    let mut value = value
        .replace("<h1>", "# ")
        .replace("</h1>", "")
        .replace("<h2>", "## ")
        .replace("</h2>", "")
        .replace("<h3>", "### ")
        .replace("</h3>", "")
        .replace("<h4>", "#### ")
        .replace("</h4>", "")
        .replace("<h5>", "##### ")
        .replace("</h5>", "")
        .replace("<h6>", "###### ")
        .replace("</h6>", "");
    // replace multiple = or - with # or ##
    let re = Regex::new(r#"^=+$"#).unwrap();
    value = re.replace_all(value.as_str(), "#").to_string();
    let re = Regex::new(r#"^-+$"#).unwrap();
    value = re.replace_all(value.as_str(), "##").to_string();

    value
}

/// Removes the HTML paragraph alignment tags from a given string.
///
/// This function replaces the '<p align="left">', '<p align="center">', '<p align="right">', '<p align="justify">', and '</p>' tags in the string with an empty string.
///
/// # Arguments
///
/// * `value` - A string that may contain HTML paragraph alignment tags which need to be removed.
///
/// # Returns
///
/// * An owned String where any HTML paragraph alignment tags have been removed.
fn remove_p_align(value: String) -> String {
    value
        .replace("<p align=\"left\">", "")
        .replace("<p align=\"center\">", "")
        .replace("<p align=\"right\">", "")
        .replace("<p align=\"justify\">", "")
        .replace("</p>", "")
}

/// Removes the image tags from a given string.
///
/// This function first removes the markdown image tags from the string.
/// It then removes the HTML image tags from the string.
/// Finally, it removes any image tags that have a number attached to them.
///
/// # Arguments
///
/// * `value` - A string that may contain image tags which need to be removed.
///
/// # Returns
///
/// * An owned String where any image tags have been removed.
fn remove_image(mut value: String) -> String {
    // remove ![*](*)
    let re = Regex::new(r#"!\[[^]]*]\([^)]*\)"#).unwrap();
    value = re.replace_all(value.as_str(), "").to_string();
    // also remove <img alt="fallback text" src="https://anilist.co/img/icons/icon.svg">
    let re = Regex::new(r#"<img[^>]*>"#).unwrap();
    value = re.replace_all(value.as_str(), "").to_string();

    // also remove img###(https://anilist.co/img/icons/icon.svg) where ### is any number
    let re = Regex::new(r#"img\d+"#).unwrap();
    value = re.replace_all(value.as_str(), "").to_string();

    value
}

/// Removes the horizontal line tags from a given string.
///
/// This function first removes the HTML horizontal line tags from the string.
/// It then removes any markdown horizontal line tags from the string.
///
/// # Arguments
///
/// * `value` - A string that may contain horizontal line tags which need to be removed.
///
/// # Returns
///
/// * An owned String where any horizontal line tags have been removed.
fn remove_horizontal_line(mut value: String) -> String {
    let re = Regex::new(r#"<hr>"#).unwrap();
    value = re.replace_all(value.as_str(), "").to_string();
    // also remove <hr />
    let re = Regex::new(r#"<hr\s*/>"#).unwrap();
    value = re.replace_all(value.as_str(), "").to_string();
    // if there is --- or *** can be 3 or more
    let re = Regex::new(r#"^-{3,}$"#).unwrap();
    value = re.replace_all(value.as_str(), "").to_string();
    let re = Regex::new(r#"^\*{3,}$"#).unwrap();
    value = re.replace_all(value.as_str(), "").to_string();

    value
}

/// Converts the HTML list tags in a given string to markdown list items.
///
/// This function first removes the '<ul>', '</ul>', '<ol>', and '</ol>' tags from the string.
/// It then replaces the '<li>' and '</li>' tags with '- '.
/// Finally, it replaces any list item that starts with '-', '*', or '+' with '-'.
///
/// # Arguments
///
/// * `value` - A string that may contain HTML list tags which need conversion.
///
/// # Returns
///
/// * An owned String where any HTML list tags have been replaced with markdown list items.
fn convert_list(value: String) -> String {
    let mut value = value
        .replace("<ul>", "")
        .replace("</ul>", "")
        .replace("<ol>", "")
        .replace("</ol>", "")
        .replace("</li>", "\\n");

    let re = Regex::new(r#"<li[^>]*>"#).unwrap();
    value = re.replace_all(value.as_str(), "- ").to_string();

    // replace single - or * or + with -
    let re = Regex::new(r#"^[-*+]"#).unwrap();
    value = re.replace_all(value.as_str(), "- ").to_string();
    value = value.replace("- -", "-");
    value = value.replace("* -", "-");
    value = value.replace("+ -", "-");
    value = value.replace("-  ", "- ");
    value
}

/// Removes the code block tags from a given string.
///
/// This function first removes the '<code>', '<pre>', '</code>', and '</pre>' tags from the string.
/// It then removes any '`' or '```' characters from the string.
///
/// # Arguments
///
/// * `value` - A string that may contain code block tags which need to be removed.
///
/// # Returns
///
/// * An owned String where any code block tags have been removed.
fn remove_code_block(value: String) -> String {
    // <code> or <pre> or </code> or </pre>
    let re = Regex::new(r#"<code[^>]*>"#).unwrap();
    let mut value = re.replace_all(value.as_str(), "").to_string();
    let re = Regex::new(r#"<pre[^>]*>"#).unwrap();
    value = re.replace_all(value.as_str(), "").to_string();
    let re = Regex::new(r#"</code>"#).unwrap();
    value = re.replace_all(value.as_str(), "").to_string();
    let re = Regex::new(r#"</pre>"#).unwrap();
    value = re.replace_all(value.as_str(), "").to_string();

    // remove ` or ```
    let re = Regex::new(r#"`"#).unwrap();
    value = re.replace_all(value.as_str(), "").to_string();
    let re = Regex::new(r#"```"#).unwrap();
    value = re.replace_all(value.as_str(), "").to_string();

    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn italic_conversion_handles_italic_tags() {
        let input = String::from("<i>Hello</i> <em>World</em>");
        let expected_output = String::from("_Hello_ _World_");
        assert_eq!(convert_italic(input), expected_output);
    }

    #[test]
    fn html_entity_conversion_handles_entities() {
        let input = String::from("Hello &mdash; World");
        let expected_output = String::from("Hello — World");
        assert_eq!(convert_html_entity_to_real_char(input), expected_output);
    }

    #[test]
    fn link_conversion_handles_html_links() {
        let input = String::from("<a href=\"https://example.com\">link</a>");
        let expected_output = String::from("[link](https://example.com)");
        assert_eq!(convert_link_to_discord_markdown(input), expected_output);
    }

    #[test]
    fn anti_slash_addition_handles_backquotes() {
        let input = String::from("Hello`World");
        let expected_output = String::from("Hello\\`World");
        assert_eq!(add_anti_slash(input), expected_output);
    }

    #[test]
    fn line_break_conversion_handles_html_line_breaks() {
        let input = String::from("Hello<br>World");
        let expected_output = String::from("Hello\nWorld");
        assert_eq!(
            convert_html_line_break_to_line_break(input),
            expected_output
        );
    }

    #[test]
    fn spoiler_conversion_handles_spoiler_tags() {
        let input = String::from("~!This is a spoiler!~");
        let expected_output = String::from("||This is a spoiler||");
        assert_eq!(convert_spoiler(input), expected_output);
    }

    #[test]
    fn bold_conversion_handles_bold_tags() {
        let input = String::from("<b>Hello</b>, <strong>World!</strong>");
        let expected_output = String::from("**Hello**, **World!**");
        assert_eq!(convert_bold(input), expected_output);
    }

    #[test]
    fn strikethrough_conversion_handles_strikethrough_tags() {
        let input = String::from("This is a ~~test~~. <del>test</del>");
        let expected_output = String::from("This is a ~~test~~. ~~test~~");
        assert_eq!(convert_strikethrough(input), expected_output);
    }

    #[test]
    fn blockquote_conversion_handles_blockquote_tags() {
        let input = String::from("<blockquote>Hello</blockquote>");
        let expected_output = String::from("> Hello");
        assert_eq!(convert_blockquote(input), expected_output);
    }

    #[test]
    fn header_conversion_handles_header_tags() {
        let input = String::from("<h1>Hello</h1>");
        let expected_output = String::from("# Hello");
        assert_eq!(convert_h_header(input), expected_output);
    }

    #[test]
    fn p_align_removal_handles_p_align_tags() {
        let input = String::from("<p align=\"left\">Hello</p>");
        let expected_output = String::from("Hello");
        assert_eq!(remove_p_align(input), expected_output);
    }

    #[test]
    fn image_removal_handles_image_tags() {
        let input = String::from("![alt text](https://example.com/image.jpg)");
        let expected_output = String::from("");
        assert_eq!(remove_image(input), expected_output);
    }

    #[test]
    fn horizontal_line_removal_handles_horizontal_line_tags() {
        let input = String::from("<hr>");
        let expected_output = String::from("");
        assert_eq!(remove_horizontal_line(input), expected_output);
    }

    #[test]
    fn list_conversion_handles_list_tags() {
        let input = String::from("<ul><li>Hello</li><li>World</li></ul>");
        let expected_output = String::from("- Hello\\n- World\\n");
        assert_eq!(convert_list(input), expected_output);
    }

    #[test]
    fn code_block_removal_handles_code_block_tags() {
        let input = String::from("<code>Hello</code>");
        let expected_output = String::from("Hello");
        assert_eq!(remove_code_block(input), expected_output);
    }

    #[test]
    fn convert_steam_to_discord_flavored_markdown_converts_italic_text() {
        let input = String::from("_italic_");
        let expected_output = String::from("_italic_");
        assert_eq!(
            convert_steam_to_discord_flavored_markdown(input),
            expected_output
        );
    }

    #[test]
    fn convert_steam_to_discord_flavored_markdown_converts_bold_text() {
        let input = String::from("**bold**");
        let expected_output = String::from("**bold**");
        assert_eq!(
            convert_steam_to_discord_flavored_markdown(input),
            expected_output
        );
    }

    #[test]
    fn convert_steam_to_discord_flavored_markdown_converts_html_entities() {
        let input = String::from("&amp;");
        let expected_output = String::from("&");
        assert_eq!(
            convert_steam_to_discord_flavored_markdown(input),
            expected_output
        );
    }

    #[test]
    fn convert_steam_to_discord_flavored_markdown_converts_links() {
        let input = String::from("<a href=\"https://example.com\">link</a>");
        let expected_output = String::from("[link](https://example.com)");
        assert_eq!(
            convert_steam_to_discord_flavored_markdown(input),
            expected_output
        );
    }
}
