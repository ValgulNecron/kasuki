use crate::command::embed_content::{
	CommandType, ComponentVersion, ComponentVersion1, EmbedsContents,
};
use crate::helper::create_default_embed::get_default_embed;
use anyhow::{Result, anyhow};
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{CommandInteraction, MessageFlags, SkuId};
use serenity::builder::{
	CreateActionRow, CreateAttachment, CreateButton, CreateComponent, CreateEmbedAuthor,
	CreateEmbedFooter, CreateInteractionResponse, CreateInteractionResponseFollowup,
	CreateInteractionResponseMessage,
};
use serenity::prelude::Context as SerenityContext;
use std::borrow::Cow;

pub trait Command {
	fn get_ctx(&self) -> &SerenityContext;

	fn get_command_interaction(&self) -> &CommandInteraction;
	async fn get_contents<'a>(&'a self) -> Result<EmbedsContents<'a>>;
}

pub trait CommandRun {
	async fn send_embed(&self, content: EmbedsContents) -> Result<()>;

	async fn defer(&self) -> Result<()>;

	async fn run_user(&self) -> Result<()>;

	async fn run_slash(&self) -> Result<()>;

	async fn run_message(&self) -> Result<()>;
}

impl<T: Command> CommandRun for T {
	async fn send_embed(&self, contents: EmbedsContents<'_>) -> Result<()> {
		let mut embeds = vec![];
		let mut has_embed = false;
		for embed_content in contents.embed_contents {
			has_embed = true;
			let mut embed = get_default_embed(embed_content.colour)
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
		let mut is_v2 = false;
		if let Some(action_row) = contents.action_row {
			match action_row {
				ComponentVersion::V1(v1) => match v1 {
					ComponentVersion1::Buttons(buttons) => {
						let mut components = vec![];
						for button in buttons {
							let mut button_builder =
								match (button.custom_id, button.url, button.sku_id) {
									(Some(id), None, None) => CreateButton::new(id),
									(None, Some(url), None) => CreateButton::new_link(url),
									(None, None, Some(sku_id)) => {
										CreateButton::new_premium(SkuId::new(sku_id.parse()?))
									},
									_ => {
										return Err(anyhow!("Invalid button"));
									},
								};
							if let Some(emoji) = button.emoji {
								button_builder = button_builder.emoji(emoji);
							}
							if let Some(style) = button.style {
								button_builder = button_builder.style(style);
							}
							button_builder = button_builder.label(button.label);
							button_builder = button_builder.disabled(button.disabled);
							components.push(button_builder)
						}
						component = Some(Cow::Owned(vec![CreateComponent::ActionRow(
							CreateActionRow::Buttons(Cow::Owned(components)),
						)]))
					},
					ComponentVersion1::SelectMenu(select_menu) => {
						return Err(anyhow!("Component V1 SelectMenu is not supported yet"));
					},
					ComponentVersion1::InputText(input_text) => {
						return Err(anyhow!("Component V1 InputText is not supported yet"));
					},
				},
				ComponentVersion::V2(v2) => {
					is_v2 = true;
					component = Some(v2.components)
				},
			};
		};

		let ctx = self.get_ctx();
		let command_interaction = self.get_command_interaction();

		match contents.command_type {
			CommandType::First => {
				let mut builder = CreateInteractionResponseMessage::new().files(files);
				if has_embed {
					builder = builder.embeds(embeds);
				} else if let Some(component) = component.clone()
					&& is_v2
				{
					builder = builder
						.components(component)
						.flags(MessageFlags::IS_COMPONENTS_V2);
				}
				if let Some(component) = component
					&& !is_v2
				{
					builder = builder.components(component);
				}
				let builder = CreateInteractionResponse::Message(builder);
				command_interaction
					.create_response(&ctx.http, builder)
					.await?;
			},
			CommandType::Followup => {
				let mut builder = CreateInteractionResponseFollowup::new().files(files);
				if has_embed {
					builder = builder.embeds(embeds);
				} else if let Some(component) = component.clone()
					&& is_v2
				{
					builder = builder
						.components(component)
						.flags(MessageFlags::IS_COMPONENTS_V2);
				}
				if let Some(component) = component
					&& !is_v2
				{
					builder = builder.components(component);
				}
				command_interaction
					.create_followup(&ctx.http, builder)
					.await?;
			},
		}

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
