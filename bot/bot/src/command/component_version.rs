use std::borrow::Cow;

#[derive(Clone)]
pub enum ComponentVersion<'a> {
	V2(ComponentVersion2<'a>),
}

#[derive(Clone)]
pub struct ComponentVersion2<'a> {
	pub components: Cow<'a, [serenity::builder::CreateComponent<'a>]>,
}
