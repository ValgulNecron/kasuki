use std::env;
use sea_orm_migration::prelude::*;
use migration::sea_orm::sqlx::{query, PgPool};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let _ = match PgPool::connect(&database_url).await {
        Ok(_) => (),
        Err(_) => {
            let database_url = database_url.replace("kasuki", "");
            let pool = PgPool::connect(&database_url).await?;
            query("CREATE DATABASE kasuki").execute(&pool).await?;
            ()
        }
    };
    cli::run_cli(migration::Migrator).await;

    Ok(())
}