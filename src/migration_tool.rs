use sqlx::migrate::Migrator;
use sqlx::PgPool;
use std::env;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set");

    let pool = PgPool::connect(&database_url).await?;

    // Run migrations
    MIGRATOR.run(&pool).await?;

    println!("Migrations applied successfully.");

    Ok(())
}
