use serenity::all::{
	ButtonStyle, ChannelType, Colour, GenericChannelId, InputTextStyle, ReactionType, RoleId,
	UserId,
};
use serenity::builder::CreateComponent;
use std::borrow::Cow;

#[derive(Clone)]
pub struct EmbedsContents<'a> {
	pub command_type: CommandType,
	pub embed_contents: Vec<EmbedContent>,
	pub action_row: Option<ComponentVersion<'a>>,
	pub files: Vec<CommandFiles>,
}

impl<'a> EmbedsContents<'a> {  
	pub fn new(command_type: CommandType, embed_contents: Vec<EmbedContent>) -> Self {
		Self {
			command_type,
			embed_contents,
			action_row: None,
			files: Vec::new(),
		}
	}

	pub fn action_row(mut self, action_row: ComponentVersion<'a>) -> Self {
		self.action_row = Some(action_row);
		self
	}

	pub fn add_file(&mut self, file: CommandFiles) -> &mut Self {
		self.files.push(file);
		self
	}

	pub fn add_embed_content(&mut self, embed_content: EmbedContent) -> &mut Self {
		self.embed_contents.push(embed_content);
		self
	}

	pub fn add_embed_contents(&mut self, embed_contents: Vec<EmbedContent>) -> &mut Self {
		self.embed_contents.extend(embed_contents);
		self
	}

	pub fn add_files(&mut self, files: Vec<CommandFiles>) -> &mut Self {
		self.files.extend(files);
		self
	}
}

#[derive(Clone)]
pub enum CommandType {
	First,
	Followup,
}

impl Default for CommandType {
	fn default() -> Self {
		Self::First
	}
}

#[derive(Clone)]
pub struct CommandFiles {
	pub filename: String,
	pub description: Option<String>,
	pub data: Vec<u8>,
}

impl CommandFiles {
	pub fn new(filename: String, data: Vec<u8>) -> Self {
		Self {
			filename,
			description: None,
			data,
		}
	}

	pub fn description(mut self, description: String) -> Self {
		self.description = Some(description);
		self
	}
}

#[derive(Clone)]
pub struct EmbedContent {
	pub title: String,
	pub description: Option<String>,
	pub thumbnail: Option<String>,
	pub url: Option<String>,
	pub colour: Option<Colour>,
	pub fields: Vec<(String, String, bool)>,
	pub images_url: Option<String>,
	pub footer: Option<CreateFooter>,
	pub author: Option<CreateAuthor>,
}

impl EmbedContent {
	pub fn new(title: String) -> Self {
		Self {
			title,
			description: None,
			thumbnail: None,
			url: None,
			colour: None,
			fields: Vec::new(),
			images_url: None,
			footer: None,
			author: None,
		}
	}

	pub fn description(mut self, description: String) -> Self {
		self.description = Some(description);
		self
	}

	pub fn thumbnail(mut self, thumbnail: String) -> Self {
		self.thumbnail = Some(thumbnail);
		self
	}

	pub fn url(mut self, url: String) -> Self {
		self.url = Some(url);
		self
	}

	pub fn colour(mut self, colour: Colour) -> Self {
		self.colour = Some(colour);
		self
	}

	pub fn fields(mut self, fields: Vec<(String, String, bool)>) -> Self {
		self.fields = fields;
		self
	}

	pub fn images_url(mut self, images_url: String) -> Self {
		self.images_url = Some(images_url);
		self
	}

	pub fn footer(mut self, footer: CreateFooter) -> Self {
		self.footer = Some(footer);
		self
	}

	pub fn author(mut self, author: CreateAuthor) -> Self {
		self.author = Some(author);
		self
	}
}

#[derive(Clone)]
pub struct CreateFooter {
	pub text: String,
	pub icon_url: Option<String>,
}

impl CreateFooter {
	pub fn new(text: String) -> Self {
		Self {
			text,
			icon_url: None,
		}
	}

	pub fn icon_url(mut self, icon_url: String) -> Self {
		self.icon_url = Some(icon_url);
		self
	}
}

#[derive(Clone)]
pub struct CreateAuthor {
	pub name: String,
	pub icon_url: Option<String>,
	pub url: Option<String>,
}

impl CreateAuthor {
	pub fn new(name: String) -> Self {
		Self {
			name,
			icon_url: None,
			url: None,
		}
	}

	pub fn icon_url(mut self, icon_url: String) -> Self {
		self.icon_url = Some(icon_url);
		self
	}

	pub fn url(mut self, url: String) -> Self {
		self.url = Some(url);
		self
	}
}

#[derive(Clone)]
pub enum ComponentVersion<'a> {
	V1(ComponentVersion1),
	V2(ComponentVersion2<'a>),
}

impl<'a> ComponentVersion<'a> {
	pub fn v1(v1: ComponentVersion1) -> Self {
		ComponentVersion::V1(v1)
	}

	pub fn v2(v2: ComponentVersion2<'a>) -> Self {
		ComponentVersion::V2(v2)
	}
}

#[derive(Clone)]
pub enum ComponentVersion1 {
	Buttons(Vec<ButtonV1>),
	SelectMenu(SelectMenuV1),
	InputText(InputTextV1),
}

impl ComponentVersion1 {
	pub fn buttons(buttons: Vec<ButtonV1>) -> Self {
		ComponentVersion1::Buttons(buttons)
	}

	pub fn select_menu(select_menu: SelectMenuV1) -> Self {
		ComponentVersion1::SelectMenu(select_menu)
	}

	pub fn input_text(input_text: InputTextV1) -> Self {
		ComponentVersion1::InputText(input_text)
	}
}

#[derive(Clone)]
pub struct ButtonV1 {
	pub label: String,
	pub style: Option<ButtonStyle>,
	pub custom_id: Option<String>,
	pub sku_id: Option<String>,
	pub emoji: Option<ReactionType>,
	pub disabled: bool,
	pub url: Option<String>,
}

impl ButtonV1 {
	pub fn new(label: String) -> Self {
		ButtonV1 {
			label,
			style: None,
			custom_id: None,
			sku_id: None,
			emoji: None,
			disabled: false,
			url: None,
		}
	}

	pub fn style(mut self, style: ButtonStyle) -> Self {
		self.style = Some(style);
		self
	}

	pub fn custom_id(mut self, custom_id: String) -> Self {
		self.custom_id = Some(custom_id);
		self
	}

	pub fn sku_id(mut self, sku_id: String) -> Self {
		self.sku_id = Some(sku_id);
		self
	}

	pub fn emoji(mut self, emoji: ReactionType) -> Self {
		self.emoji = Some(emoji);
		self
	}

	pub fn disabled(mut self, disabled: bool) -> Self {
		self.disabled = disabled;
		self
	}

	pub fn url(mut self, url: String) -> Self {
		self.url = Some(url);
		self
	}
}

#[derive(Clone)]
pub struct SelectMenuV1 {
	pub placeholder: String,
	pub custom_id: String,
	pub min_values: Option<u8>,
	pub max_values: Option<u8>,
	pub disabled: Option<bool>,
	pub select_menu_kind: SelectMenuKindV1,
}

impl SelectMenuV1 {
	pub fn new(placeholder: String, select_menu_kind: SelectMenuKindV1, custom_id: String) -> Self {
		SelectMenuV1 {
			placeholder,
			custom_id,
			min_values: None,
			max_values: None,
			disabled: None,
			select_menu_kind,
		}
	}

	pub fn min_values(mut self, min_values: u8) -> Self {
		self.min_values = Some(min_values);
		self
	}

	pub fn max_values(mut self, max_values: u8) -> Self {
		self.max_values = Some(max_values);
		self
	}

	pub fn disabled(mut self, disabled: bool) -> Self {
		self.disabled = Some(disabled);
		self
	}
}

#[derive(Clone)]
pub enum SelectMenuKindV1 {
	String(Vec<CreateSelectMenuOptionV1>),
	User(Vec<UserId>),
	Role(Vec<RoleId>),
	Mentionable {
		default_users: Vec<UserId>,
		default_roles: Vec<RoleId>,
	},
	Channel {
		channel_types: Vec<ChannelType>,
		default_channels: Vec<GenericChannelId>,
	},
}

#[derive(Clone)]
pub struct CreateSelectMenuOptionV1 {
	label: String,
	value: String,
	description: Option<String>,
	emoji: Option<ReactionType>,
	default: Option<bool>,
}

#[derive(Clone)]
pub struct InputTextV1 {
	custom_id: String,
	style: InputTextStyle,
	label: Option<String>,
	min_length: Option<u16>,
	max_length: Option<u16>,
	required: bool,
	value: Option<String>,
	placeholder: Option<String>,
}

#[derive(Clone)]
pub struct ComponentVersion2<'a> {
	pub components: Cow<'a, CreateComponent<'a>>,
}
