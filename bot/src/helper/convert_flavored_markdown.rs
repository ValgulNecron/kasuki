use markdown_converter::anilist::convert_anilist_flavored_markdown;
use markdown_converter::steam::convert_steam_flavored_markdown;

pub fn convert_anilist_flavored_to_discord_flavored_markdown(value: String) -> String {
    convert_anilist_flavored_markdown(value.as_str()).to_string()
}

pub fn convert_steam_to_discord_flavored_markdown(value: String) -> String {
    convert_steam_flavored_markdown(value.as_str()).to_string()
}
