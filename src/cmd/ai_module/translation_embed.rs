use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::model::Timestamp;

pub async fn translation_embed(ctx: &Context, text: &str, message: serenity::Result<Message>) {
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