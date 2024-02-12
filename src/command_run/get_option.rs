use serenity::all::CommandInteraction;
use std::collections::HashMap;

pub fn get_option_map(interaction: &CommandInteraction) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for option in &interaction.data.options {
        let search = option.value.as_str().unwrap_or_default().to_string();
        let name = option.name.clone();
        map.insert(name, search);
    }

    map
}
