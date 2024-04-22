use crate::common::anilist_to_discord_markdown::{
    add_anti_slash, convert_blockquote, convert_bold, convert_h_header,
    convert_html_entity_to_real_char, convert_html_line_break_to_line_break, convert_italic,
    convert_link_to_discord_markdown, convert_strikethrough,
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
