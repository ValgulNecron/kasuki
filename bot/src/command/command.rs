use crate::command::embed_content::{CommandType, ComponentTypeV1, ComponentVersion, ComponentVersion1, EmbedContent, EmbedsContents};
use anyhow::{anyhow, Result};
use serenity::all::{Button, CommandInteraction, ComponentType};
use serenity::all::CreateInteractionResponse::Defer;
use serenity::builder::{CreateAttachment, CreateButton, CreateEmbedAuthor, CreateEmbedFooter, CreateInteractionResponse, CreateInteractionResponseFollowup, CreateInteractionResponseMessage, CreateMessage};
use serenity::prelude::Context as SerenityContext;
use crate::helper::create_default_embed::get_default_embed;

pub trait Command {
	fn get_ctx(&self) -> &SerenityContext;

	fn get_command_interaction(&self) -> &CommandInteraction;

	async fn get_contents(&self) -> Result<EmbedsContents>;
}

pub trait CommandRun {
	async fn send_embed(&self, content: EmbedsContents) -> Result<()>;

	async fn defer(&self) -> Result<()>;

	async fn run_user(&self) -> Result<()>;

	async fn run_slash(&self) -> Result<()>;

	async fn run_message(&self) -> Result<()>;
}

impl<T: Command> CommandRun for T {
	async fn send_embed(&self, contents: EmbedsContents) -> Result<()> {
		let mut embeds = vec![];
		for embed_content in contents.embed_contents {
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
		
		let mut components = vec![];
		for component in contents.action_row {
			let mut component_builder = match component{
				ComponentVersion::V1(v1) => {
					let mut rows_builder = vec![];
					for rows in v1.action_row {
						let mut row_builder = vec![];
						for row in rows {
							match row {
								ComponentTypeV1::Button(button) => {
									match (button.url, button.custom_id, button.sku_id ){
										(Some(url), None, None) => {
											let mut button_builder = CreateButton::new_link(url)
												.label(button.label)
												.disabled(button.disabled);
											if let Some(emoji) = button.emoji {
												button_builder = button_builder.emoji(emoji);
											}
											if let Some(style) = button.style {
												button_builder = button_builder.style(style);
											}
											
											row_builder.push(button_builder);
										}
										(None, Some(id), None) => {}
										(None, None, Some(sku_id)) => {}
									}
								}
								ComponentTypeV1::SelectMenu(select_menu) => {}
								ComponentTypeV1::InputText(input_text) => {}
							}
						}
					}
				}
				ComponentVersion::V2(v2) => {
					return Err(anyhow!("Component V2 is not supported yet"));
				}
			};
		}
		
		
		let ctx = self.get_ctx();
		let command_interaction = self.get_command_interaction();
		
		match contents.command_type {
			CommandType::First => {	
				let builder = CreateInteractionResponseMessage::new()
					.embeds(embeds)
					.files(files)
					.components(components);
				let builder = CreateInteractionResponse::Message(builder);
				command_interaction.create_response(&ctx.http, builder).await?;
			}
			CommandType::Followup => {
				let builder = CreateInteractionResponseFollowup::new()
					.embeds(embeds)
					.files(files)
					.components(components);
				command_interaction.create_followup(&ctx.http, builder).await?;
			}
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
