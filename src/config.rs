use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::auth::Tokens;
use crate::data::DashboardData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
}

impl Config {
    pub fn load() -> Self {
        // Try to load from environment variables first
        let client_id = std::env::var("WHOOP_CLIENT_ID").ok();
        let client_secret = std::env::var("WHOOP_CLIENT_SECRET").ok();

        Config {
            client_id,
            client_secret,
        }
    }

    pub fn data_dir() -> Result<PathBuf> {
        let dir = dirs::data_dir()
            .context("Failed to get data directory")?
            .join("whoop-cli");
        
        fs::create_dir_all(&dir)?;
        Ok(dir)
    }

    pub fn save_tokens(&self, tokens: &Tokens) -> Result<()> {
        let path = Self::data_dir()?.join("tokens.json");
        let json = serde_json::to_string_pretty(tokens)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load_tokens(&self) -> Result<Tokens> {
        let path = Self::data_dir()?.join("tokens.json");
        let json = fs::read_to_string(path)?;
        let tokens: Tokens = serde_json::from_str(&json)?;
        Ok(tokens)
    }

    pub fn save_cache(&self, data: &DashboardData) -> Result<()> {
        let path = Self::data_dir()?.join("cache.json");
        let json = serde_json::to_string_pretty(data)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load_cache(&self) -> Result<DashboardData> {
        let path = Self::data_dir()?.join("cache.json");
        let json = fs::read_to_string(path)?;
        let data: DashboardData = serde_json::from_str(&json)?;
        Ok(data)
    }
}
