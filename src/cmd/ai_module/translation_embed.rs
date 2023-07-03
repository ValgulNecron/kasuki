use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::model::Timestamp;
use serenity::utils::Colour;

pub async fn translation_embed(ctx: &Context, text: String, message: serenity::Result<Message>) {
    let color = Colour::FABLED_PINK;
    let mut real_message = message.unwrap();
    real_message.edit(&ctx.http, |m|
        m.embed((|e| {
            e.title("Here your translation")
                .description(format!("{}", text))
                .timestamp(Timestamp::now())
                .color(color)
        })
        )).await.expect("TODO");
}