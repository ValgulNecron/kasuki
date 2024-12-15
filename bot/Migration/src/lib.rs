pub use sea_orm_migration::prelude::*;

mod m20240815_180000_guild_data;
mod m20240815_180201_user_data;
mod m20240815_180308_server_image;
mod m20240815_182736_user_color;
mod m20240815_183343_registered_anilist_user;
mod m20240815_190656_module_activation;
mod m20240815_213206_guild_lang;
mod m20240815_231355_kill_switch;
mod m20240815_231524_activity_data;
mod m20240815_231538_ping_history;
mod m20240826_215627_server_user_relation;
mod m20240831_133253_user_subscription;
mod m20240831_134027_guild_subscription;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
	fn migrations() -> Vec<Box<dyn MigrationTrait>> {
		vec![
			Box::new(m20240815_180000_guild_data::Migration),
			Box::new(m20240815_180201_user_data::Migration),
			Box::new(m20240815_180308_server_image::Migration),
			Box::new(m20240815_182736_user_color::Migration),
			Box::new(m20240815_183343_registered_anilist_user::Migration),
			Box::new(m20240815_190656_module_activation::Migration),
			Box::new(m20240815_213206_guild_lang::Migration),
			Box::new(m20240815_231355_kill_switch::Migration),
			Box::new(m20240815_231524_activity_data::Migration),
			Box::new(m20240815_231538_ping_history::Migration),
			Box::new(m20240826_215627_server_user_relation::Migration),
			Box::new(m20240831_133253_user_subscription::Migration),
			Box::new(m20240831_134027_guild_subscription::Migration),
		]
	}
}
