use crate::command::component_version::ComponentVersion;

#[derive(Clone)]
pub struct EmbedsContents<'a> {
	pub embed_contents: Vec<EmbedContent>,
	pub action_row: Option<ComponentVersion<'a>>,
	pub files: Vec<CommandFiles>,
}

impl<'a> EmbedsContents<'a> {
	pub fn new(embed_contents: Vec<EmbedContent>) -> Self {
		Self {
			embed_contents,
			action_row: None,
			files: Vec::new(),
		}
	}

	pub fn action_row(mut self, action_row: ComponentVersion<'a>) -> Self {
		self.action_row = Some(action_row);
		self
	}

	pub fn add_files(mut self, files: Vec<CommandFiles>) -> Self {
		self.files.extend(files);
		self
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
}

#[derive(Clone)]
pub struct EmbedContent {
	pub title: String,
	pub description: Option<String>,
	pub thumbnail: Option<String>,
	pub url: Option<String>,
	pub colour: Option<u32>,
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

	pub fn colour(mut self, colour: u32) -> Self {
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
}

#[derive(Clone)]
pub struct CreateAuthor {
	pub name: String,
	pub icon_url: Option<String>,
}

impl CreateAuthor {}
