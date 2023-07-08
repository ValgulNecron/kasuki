use serenity::utils::Colour;

use crate::cmd::anilist_module::struct_user::*;

pub fn get_user_color(data: UserData) -> Colour {
    let mut _color = Colour::FABLED_PINK;
    match data
        .data
        .user
        .options
        .profile_color
        .unwrap_or_else(|| "#FF00FF".to_string())
        .as_str()
    {
        "blue" => _color = Colour::BLUE,
        "purple" => _color = Colour::PURPLE,
        "pink" => _color = Colour::MEIBE_PINK,
        "orange" => _color = Colour::ORANGE,
        "red" => _color = Colour::RED,
        "green" => _color = Colour::DARK_GREEN,
        "gray" => _color = Colour::LIGHT_GREY,
        _ => {
            _color = {
                let hex_code = "#0D966D";
                let color_code = u32::from_str_radix(&hex_code[1..], 16).unwrap();
                Colour::new(color_code)
            }
        }
    }
    _color
}
