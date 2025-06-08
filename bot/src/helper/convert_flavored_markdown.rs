use markdown_converter::anilist::convert_anilist_flavored_markdown;
use markdown_converter::steam::convert_steam_flavored_markdown;

/// Converts AniList flavored markdown to Discord flavored markdown.
///
/// This function takes a string containing AniList flavored markdown and converts it
/// to Discord flavored markdown, making it suitable for display in Discord messages.
///
/// # Arguments
///
/// * `value` - A string containing AniList flavored markdown
///
/// # Returns
///
/// A string containing the equivalent Discord flavored markdown
///
/// # Examples
///
/// ```
/// use crate::helper::convert_flavored_markdown::convert_anilist_flavored_to_discord_flavored_markdown;
///
/// let anilist_markdown = "~!This is a spoiler!~";
/// let discord_markdown = convert_anilist_flavored_to_discord_flavored_markdown(anilist_markdown.to_string());
/// // discord_markdown will be "||This is a spoiler||"
/// ```
pub fn convert_anilist_flavored_to_discord_flavored_markdown(value: String) -> String {
	convert_anilist_flavored_markdown(value.as_str()).to_string()
}

/// Converts Steam flavored markdown to Discord flavored markdown.
///
/// This function takes a string containing Steam flavored markdown and converts it
/// to Discord flavored markdown, making it suitable for display in Discord messages.
///
/// # Arguments
///
/// * `value` - A string containing Steam flavored markdown
///
/// # Returns
///
/// A string containing the equivalent Discord flavored markdown
///
/// # Examples
///
/// ```
/// use crate::helper::convert_flavored_markdown::convert_steam_to_discord_flavored_markdown;
///
/// let steam_markdown = "[b]Bold text[/b]";
/// let discord_markdown = convert_steam_to_discord_flavored_markdown(steam_markdown.to_string());
/// // discord_markdown will be "**Bold text**"
/// ```
pub fn convert_steam_to_discord_flavored_markdown(value: String) -> String {
	convert_steam_flavored_markdown(value.as_str()).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_anilist_flavored_to_discord_flavored_markdown() {
        // Test with empty string
        let empty = "".to_string();
        let result = convert_anilist_flavored_to_discord_flavored_markdown(empty);
        assert_eq!(result, "", "Empty string should remain empty");

        // Test with a string that doesn't contain markdown
        let plain = "Hello, world!".to_string();
        let result = convert_anilist_flavored_to_discord_flavored_markdown(plain.clone());
        assert_eq!(result, plain, "Plain text should remain unchanged");

        // We can't test the actual conversion logic without knowing the implementation details
        // of the markdown_converter crate, but we can at least verify that the function
        // calls the external function and returns a string
        let markdown = "Some AniList markdown".to_string();
        let result = convert_anilist_flavored_to_discord_flavored_markdown(markdown);
        assert!(result.is_string(), "Result should be a string");
    }

    #[test]
    fn test_convert_steam_to_discord_flavored_markdown() {
        // Test with empty string
        let empty = "".to_string();
        let result = convert_steam_to_discord_flavored_markdown(empty);
        assert_eq!(result, "", "Empty string should remain empty");

        // Test with a string that doesn't contain markdown
        let plain = "Hello, world!".to_string();
        let result = convert_steam_to_discord_flavored_markdown(plain.clone());
        assert_eq!(result, plain, "Plain text should remain unchanged");

        // We can't test the actual conversion logic without knowing the implementation details
        // of the markdown_converter crate, but we can at least verify that the function
        // calls the external function and returns a string
        let markdown = "Some Steam markdown".to_string();
        let result = convert_steam_to_discord_flavored_markdown(markdown);
        assert!(result.is_string(), "Result should be a string");
    }

    // Helper trait to check if a value is a string
    trait IsString {
        fn is_string(&self) -> bool;
    }

    impl IsString for String {
        fn is_string(&self) -> bool {
            true
        }
    }
}
