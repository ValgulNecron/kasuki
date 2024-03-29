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

    result = convert_html_entity_to_real_char(result);
    result = convert_link_to_discord_markdown(result);
    result = add_anti_slash(result);
    result = convert_html_line_break_to_line_break(result);
    result = convert_spoiler(result);
    result = convert_h_header(result);

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

/// This function takes an input string and replaces all occurrences of the HTML entity "&mdash;" with its equivalent symbol "—".
///
/// # Arguments
///
/// * `value` - The input string which can potentially contain HTML entities.
///
/// # Returns
///
/// A new string where all occurrences of "&mdash;" are replaced with "—".
///
/// # Examples
///
/// ```
/// let input = "Hello &mdash; World".to_string();
/// let output = convert_html_entity_to_real_char(input);
/// assert_eq!(output, "Hello — World");
/// ```
pub fn convert_html_entity_to_real_char(value: String) -> String {
    value.replace("&mdash;", "—")
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

pub fn convert_blockquote(value: String) -> String {
    value
        .replace("<blockquote>", "> ")
        .replace("</blockquote>", "> ")
}

pub fn convert_h_header(value: String) -> String {
    value
        .replace("<h1>", "# ")
        .replace("</h1>", " ")
        .replace("<h2>", "## ")
        .replace("</h2>", " ")
        .replace("<h3>", "### ")
        .replace("</h3>", " ")
        .replace("<h4>", "#### ")
        .replace("</h4>", " ")
        .replace("<h5>", "##### ")
        .replace("</h5>", " ")
        .replace("<h6>", "###### ")
        .replace("</h6>", " ")
}

pub fn remove_p_align(value: String) -> String {
    value
        .replace("<p align=\"left\">", "")
        .replace("<p align=\"center\">", "")
        .replace("<p align=\"right\">", "")
        .replace("<p align=\"justify\">", "")
        .replace("</p>", "")
}
