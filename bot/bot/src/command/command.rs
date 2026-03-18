use crate::command::component_version::ComponentVersion;
use crate::command::embed_content::EmbedsContents;
use crate::helper::create_default_embed::get_default_embed;
use anyhow::Result;
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{CommandInteraction, MessageFlags};
use serenity::builder::{
	CreateAttachment, CreateEmbedAuthor, CreateEmbedFooter, CreateInteractionResponseFollowup,
};
use serenity::prelude::Context as SerenityContext;

pub trait Command {
	fn get_ctx(&self) -> &SerenityContext;

	fn get_command_interaction(&self) -> &CommandInteraction;
	async fn get_contents<'a>(&'a self) -> Result<EmbedsContents<'a>>;
}

#[macro_export]
macro_rules! impl_command {
	(
        for $type:ty,
        get_contents = $get_contents_fn:expr
    ) => {
		impl Command for $type {
			fn get_ctx(&self) -> &SerenityContext {
				&self.ctx
			}
			fn get_command_interaction(&self) -> &CommandInteraction {
				&self.command_interaction
			}
			async fn get_contents<'a>(&'a self) -> anyhow::Result<EmbedsContents<'a>> {
				($get_contents_fn)(self.clone()).await
			}
		}
	};
}

pub trait CommandRun {
	async fn send_embed(&self, content: EmbedsContents) -> Result<()>;

	async fn defer(&self) -> Result<()>;

	async fn run_user(&self) -> Result<()>;

	async fn run_slash(&self) -> Result<()>;
}

impl<T: Command> CommandRun for T {
	async fn send_embed(&self, contents: EmbedsContents<'_>) -> Result<()> {
		let mut embeds = vec![];
		let mut has_embed = false;
		for embed_content in contents.embed_contents {
			has_embed = true;
			let user_color = &self.get_command_interaction().user.accent_colour;
			let mut embed = get_default_embed(embed_content.colour, user_color)
				.title(embed_content.title)
				.fields(embed_content.fields);

			if let Some(desc) = embed_content.description {
				embed = embed.description(desc);
			}
			if let Some(thumbnail) = embed_content.thumbnail {
				embed = embed.thumbnail(thumbnail);
			}
			if let Some(url) = embed_content.url {
				embed = embed.url(url);
			}
			if let Some(images_url) = embed_content.images_url {
				embed = embed.image(images_url);
			}
			if let Some(footer) = embed_content.footer {
				let mut footer_builder = CreateEmbedFooter::new(footer.text);
				if let Some(icon_url) = footer.icon_url {
					footer_builder = footer_builder.icon_url(icon_url);
				}
				embed = embed.footer(footer_builder);
			}
			if let Some(author) = embed_content.author {
				let mut author_footer = CreateEmbedAuthor::new(author.name);
				if let Some(icon_url) = author.icon_url {
					author_footer = author_footer.icon_url(icon_url);
				}
				embed = embed.author(author_footer);
			}

			embeds.push(embed);
		}

		let mut files = vec![];
		for file in contents.files {
			let mut file_builder = CreateAttachment::bytes(file.data, file.filename);
			if let Some(desc) = file.description {
				file_builder = file_builder.description(desc);
			}
			files.push(file_builder);
		}

		let mut component = None;
		if let Some(action_row) = contents.action_row {
			match action_row {
				ComponentVersion::V2(v2) => component = Some(v2.components),
			};
		};

		let ctx = self.get_ctx();
		let command_interaction = self.get_command_interaction();

		let mut builder = CreateInteractionResponseFollowup::new().files(files);
		if has_embed {
			builder = builder.embeds(embeds);
		} else if let Some(component) = component {
			builder = builder
				.components(component)
				.flags(MessageFlags::IS_COMPONENTS_V2);
		}
		command_interaction
			.create_followup(&ctx.http, builder)
			.await?;

		Ok(())
	}

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
		self.defer().await?;
		let embed_contents = self.get_contents().await?;
		self.send_embed(embed_contents).await
	}

	async fn run_slash(&self) -> Result<()> {
		self.defer().await?;
		let embed_contents = self.get_contents().await?;
		self.send_embed(embed_contents).await
	}
}
