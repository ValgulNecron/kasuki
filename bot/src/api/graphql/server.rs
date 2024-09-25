use crate::api::graphql::query_root;
use crate::config::DbConfig;
use crate::get_url;
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_poem::GraphQL;
use lazy_static::lazy_static;
use poem::{get, handler, listener::TcpListener, web::Html, IntoResponse, Route, Server};
use sea_orm::Database;
use std::env;
use tracing::{error, info};

lazy_static! {
    static ref URL: String = env::var("URL").unwrap_or("localhost:22540".into());
    static ref ENDPOINT: String = env::var("ENDPOINT").unwrap_or("/".into());
    static ref DEPTH_LIMIT: Option<usize> = env::var("DEPTH_LIMIT").map_or(None, |data| Some(
        data.parse().expect("DEPTH_LIMIT is not a number")
    ));
    static ref COMPLEXITY_LIMIT: Option<usize> =
        env::var("COMPLEXITY_LIMIT").map_or(None, |data| {

            Some(data.parse().expect("COMPLEXITY_LIMIT is not a number"))
        });
}

#[handler]

async fn graphql_playground() -> impl IntoResponse {

    Html(playground_source(GraphQLPlaygroundConfig::new(&ENDPOINT)))
}

pub async fn launch(db_config: DbConfig) {

    dotenvy::dotenv().ok();

    let database = match Database::connect(get_url(db_config)).await {
        Ok(database) => database,
        Err(e) => {

            error!("Failed to connect to database: {:?}", e);

            return;
        }
    };

    let schema = match query_root::schema(database, *DEPTH_LIMIT, *COMPLEXITY_LIMIT) {
        Ok(schema) => schema,
        Err(e) => {

            error!("Failed to connect to database: {:?}", e);

            return;
        }
    };

    let app = Route::new().at(
        &*ENDPOINT,
        get(graphql_playground).post(GraphQL::new(schema)),
    );

    info!("Visit GraphQL Playground at http://{}", *URL);

    match Server::new(TcpListener::bind(&*URL)).run(app).await {
        Ok(server) => server,
        Err(e) => {

            error!("Failed to connect to database: {:?}", e);

            return;
        }
    };
}
