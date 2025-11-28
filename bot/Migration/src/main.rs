use migration::sea_orm::sqlx::{query, PgPool};
use sea_orm_migration::prelude::*;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
	println!("Database url: {}", database_url);
	let db_name = database_url
		.split("/")
		.last()
		.unwrap_or_default()
		.split("?")
		.collect::<Vec<&str>>()[0];
	println!("db_name: {}", db_name);
	match PgPool::connect(&database_url).await {
		Ok(_) => (),
		Err(_) => {
			let database_url = database_url.replace(db_name, "");
			let pool = PgPool::connect(&database_url).await?;
			query(&format!("CREATE DATABASE {}", db_name))
				.execute(&pool)
				.await?;
		},
	};
	cli::run_cli(migration::Migrator).await;
	print!("\x1B[2J\x1B[1;1H");
	Ok(())
}
