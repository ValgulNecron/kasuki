use serenity::all::CommandInteraction;

pub async fn autocomplete_dispatching(command: CommandInteraction) {
    match command.data.name.as_str() {
        _ => {}
    }
}
