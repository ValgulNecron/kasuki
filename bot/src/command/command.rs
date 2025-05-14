use crate::command::embed_content::EmbedContent;
use anyhow::Result;
use serenity::all::CommandInteraction;
use serenity::all::CreateInteractionResponse::Defer;
use serenity::prelude::Context as SerenityContext;

pub trait Command {
	fn get_ctx(&self) -> &SerenityContext;

	fn get_command_interaction(&self) -> &CommandInteraction;

	async fn get_contents(&self) -> Result<Vec<EmbedContent>>;
}

pub trait CommandRun {
	async fn send_embed(&self, content: Vec<EmbedContent>) -> Result<()>;

	async fn defer(&self) -> Result<()>;

	async fn run_user(&self) -> Result<()>;

	async fn run_slash(&self) -> Result<()>;

	async fn run_message(&self) -> Result<()>;
}

impl<T: Command> CommandRun for T {
	async fn send_embed(&self, contents: Vec<EmbedContent>) -> Result<()> {}

	async fn defer(&self) -> Result<()> {
		let ctx = self.get_ctx();

		let command_interaction = self.get_command_interaction();

		let builder_message = Defer(Default::default());

		command_interaction
			.create_response(&ctx.http, builder_message)
			.await?;

		Ok(())
	}

	async fn run_user(&self) -> Result<()> {
		let embed_contents = self.get_contents().await?;
		self.send_embed(embed_contents).await
	}

	async fn run_slash(&self) -> Result<()> {
		let embed_contents = self.get_contents().await?;
		self.send_embed(embed_contents).await
	}

	async fn run_message(&self) -> Result<()> {
		let embed_contents = self.get_contents().await?;
		self.send_embed(embed_contents).await
	}
}
