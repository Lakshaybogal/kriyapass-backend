use dotenvy::dotenv;
use sqlx::{
    postgres::{PgPoolOptions, Postgres},
    Pool,
};
use std::env;
pub async fn connect_database() -> Pool<Postgres> {
    dotenv().ok();
    // Get the database URL from environment variable
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
   
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to create pool");

    println!("Successfully connected to the database!");
    println!("Server started successfully");
    pool
}
