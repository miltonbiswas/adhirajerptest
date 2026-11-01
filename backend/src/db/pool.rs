use anyhow::{Context, Result};
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, postgres::PgConnectOptions};
use std::time::Duration;

use crate::config::Config;

pub async fn create_pool(config: &Config) -> Result<PgPool> {
    let options = PgConnectOptions::new()
        .host(&config.db_host)
        .port(config.db_port)
        .database(&config.db_name)
        .username(&config.db_user)
        .password(&config.db_password);

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(10))
        .connect_with(options)
        .await
        .context("Failed to connect to PostgreSQL")?;

    sqlx::query("SELECT 1")
        .execute(&pool)
        .await
        .context("PostgreSQL connection test failed")?;

    // Apply pending database migrations.
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("Failed to apply database migrations")?;

    tracing::info!("Database connected and migrations applied successfully.");

    Ok(pool)
}
