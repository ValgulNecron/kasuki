use crate::common::anilist_to_discord_markdown::{
    add_anti_slash, convert_html_entity_to_real_char, convert_italic,
    convert_link_to_discord_markdown,
};

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
        .replace("<del>", "__")
        .replace("</del>", "__")
        .replace("<strike>", "__")
        .replace("</strike>", "__")
        .replace("~~", "__")
}

pub fn convert_blockquote(value: String) -> String {
    value
        .replace("<blockquote>", "> ")
        .replace("</blockquote>", "")
}

pub fn convert_h_header(value: String) -> String {
    value
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
        .replace("</h6>", "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_antislash() {
        let value = String::from("Hello * World");
        let result = add_anti_slash(value);
        assert_eq!(result, "Hello \\* World")
    }

    #[test]
    fn test_convert_italic() {
        let value = String::from("<i>Hello</i> <em>World</em>");
        let result = convert_italic(value);
        assert_eq!(result, "_Hello_ _World_")
    }

    #[test]
    fn test_convert_html_entity_to_real_char() {
        let value = String::from("Hello &mdash; World");
        let result = convert_html_entity_to_real_char(value);
        assert_eq!(result, "Hello — World")
    }

    #[test]
    fn test_convert_link_to_discord_markdown() {
        let value = String::from("<a href=\"https://example.com\">Example</a>");
        let result = convert_link_to_discord_markdown(value);
        assert_eq!(result, "[Example](https://example.com)")
    }

    #[test]
    fn test_convert_html_line_break_to_line_break() {
        let value = String::from("Hello<br>World");
        let result = convert_html_line_break_to_line_break(value);
        assert_eq!(result, "Hello\nWorld")
    }

    #[test]
    fn test_convert_bold() {
        let value = String::from("<b>Hello</b>, <strong>World!</strong>");
        let result = convert_bold(value);
        assert_eq!(result, "**Hello**, **World!**")
    }

    #[test]
    fn test_convert_strikethrough() {
        let value = String::from("This is a ~~test~~.");
        let result = convert_strikethrough(value);
        assert_eq!(result, "This is a __test__.")
    }

    #[test]
    fn test_convert_quote() {
        let value = String::from("<blockquote>Hello</blockquote>");
        let result = convert_blockquote(value);
        assert_eq!(result, "> Hello")
    }

    #[test]
    fn test_convert_h_header() {
        let value = String::from("<h1>Hello</h1>");
        let result = convert_h_header(value);
        assert_eq!(result, "# Hello")
    }

    #[test]
    fn test_convert_steam_to_discord_flavored_markdown() {
        let value = String::from("<i>Hello</i> <em>World</em> <b>Bold</b> <strong>Strong</strong> <del>Del</del> <strike>Strike</strike> <blockquote>Quote</blockquote> <h1>Header1</h1> <a href=\"https://example.com\">Link</a>");
        let result = convert_steam_to_discord_flavored_markdown(value);
        assert_eq!(result, "_Hello_ _World_ **Bold** **Strong** __Del__ __Strike__ > Quote # Header1 [Link](https://example.com)")
    }
}
