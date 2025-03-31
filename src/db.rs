use sea_orm::{Database, DatabaseConnection};
use std::env;

pub async fn connect_database() -> DatabaseConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    Database::connect(&database_url)
        .await
        .expect("Failed to connect to database")
}
