use serenity::all::CommandDataOption;

pub fn get_option(options: &[CommandDataOption]) -> String {
    let mut value = String::new();
    for option_data in options {
        if option_data.name.as_str() != "type" {
            let option_value = option_data.value.as_str().unwrap();
            value = option_value.to_string().clone()
        }
    }
    value
}
