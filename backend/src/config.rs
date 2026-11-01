use anyhow::{Context, Result};
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub db_host: String,
    pub db_port: u16,
    pub db_name: String,
    pub db_user: String,
    pub db_password: String,
    pub server_host: String,
    pub server_port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let db_host = required_var("DB_HOST")?;
        let db_port = required_var("DB_PORT")?
            .parse::<u16>()
            .context("DB_PORT must be a valid port number")?;
        let db_name = required_var("DB_NAME")?;
        let db_user = required_var("DB_USER")?;
        let db_password = required_var("DB_PASSWORD")?;

        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .context("SERVER_PORT must be a valid port number")?;

        Ok(Self {
            db_host,
            db_port,
            db_name,
            db_user,
            db_password,
            server_host,
            server_port,
        })
    }
}

fn required_var(name: &str) -> Result<String> {
    let value = env::var(name)
        .with_context(|| format!("Required environment variable {name} is missing"))?;

    if value.trim().is_empty() {
        anyhow::bail!("Environment variable {name} cannot be empty");
    }

    Ok(value)
}
