use crate::structure::database::*;
use async_graphql::dynamic::*;
use sea_orm::DatabaseConnection;
use seaography::{Builder, BuilderContext};

lazy_static::lazy_static! { static ref CONTEXT : BuilderContext = BuilderContext :: default () ; }

pub fn schema(
    database: DatabaseConnection,
    depth: Option<usize>,
    complexity: Option<usize>,
) -> Result<Schema, SchemaError> {

    let mut builder = Builder::new(&CONTEXT, database.clone());

    seaography::register_entities!(
        builder,
        [
            activity_data,
            guild_data,
            guild_lang,
            guild_subscription,
            kill_switch,
            module_activation,
            ping_history,
            registered_user,
            server_image,
            server_user_relation,
            user_color,
            user_data,
            user_subscription
        ]
    );

    let schema = builder.schema_builder();

    let schema = if let Some(depth) = depth {

        schema.limit_depth(depth)
    } else {

        schema
    };

    let schema = if let Some(complexity) = complexity {

        schema.limit_complexity(complexity)
    } else {

        schema
    };

    schema.data(database).finish()
}
