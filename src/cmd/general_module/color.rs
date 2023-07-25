use serenity::utils::Colour;

use crate::cmd::anilist_module::struct_user::*;

pub fn get_user_color(data: UserWrapper) -> Colour {
    let mut color = Colour::FABLED_PINK;
    match data
        .data
        .user
        .options
        .profile_color
        .unwrap_or_else(|| "#FF00FF".to_string())
        .as_str()
    {
        "blue" => color = Colour::BLUE,
        "purple" => color = Colour::PURPLE,
        "pink" => color = Colour::MEIBE_PINK,
        "orange" => color = Colour::ORANGE,
        "red" => color = Colour::RED,
        "green" => color = Colour::DARK_GREEN,
        "gray" => color = Colour::LIGHT_GREY,
        _ => {
            color = {
                let hex_code = "#0D966D";
                let color_code = u32::from_str_radix(&hex_code[1..], 16).unwrap();
                Colour::new(color_code)
            }
        }
    }
    color
}
