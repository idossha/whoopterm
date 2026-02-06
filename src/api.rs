use anyhow::{Context, Result};
use reqwest::header::{AUTHORIZATION};
use serde_json::Value;
use std::time::Duration;

use crate::config::Config;
use crate::data::{DashboardData, Profile, Recovery, Sleep, Workout};
use crate::auth::AuthManager;

const API_BASE: &str = "https://api.prod.whoop.com/developer";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

pub struct WhoopAPI {
    client: reqwest::Client,
    config: Config,
    auth: AuthManager,
}

impl WhoopAPI {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            config: Config::load(),
            auth: AuthManager::new(),
        }
    }

    pub async fn authenticate(&self) -> Result<()> {
        self.auth.authenticate().await
    }

    pub async fn test_connection(&self) -> Result<()> {
        let token = self.auth.get_access_token().await?;
        let url = format!("{}/v2/user/profile/basic", API_BASE);
        
        let response = self.client
            .get(&url)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("API test failed: {}", response.status()))
        }
    }

    pub async fn refresh_all_data(&self) -> Result<DashboardData> {
        let token = self.auth.get_access_token().await?;
        
        let profile = self.get_profile(&token).await?;
        let recovery = self.get_recovery(&token).await?;
        let sleep = self.get_sleep(&token).await?;
        let workouts = self.get_workouts(&token).await?;

        let data = DashboardData {
            profile: Some(profile),
            recovery,
            sleep,
            workouts,
            refreshed_at: Some(chrono::Utc::now()),
        };

        self.config.save_cache(&data)?;
        Ok(data)
    }

    pub async fn load_cached_or_refresh(&self) -> Result<DashboardData> {
        if let Ok(cached) = self.config.load_cache() {
            if let Some(refreshed_at) = cached.refreshed_at {
                let age = chrono::Utc::now().signed_duration_since(refreshed_at);
                if age.num_seconds() < 3600 { // 1 hour cache
                    return Ok(cached);
                }
            }
        }

        self.refresh_all_data().await
    }

    async fn get_profile(&self, token: &str) -> Result<Profile> {
        let url = format!("{}/v2/user/profile/basic", API_BASE);
        
        let response = self.client
            .get(&url)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .send()
            .await?;

        let profile = response.json::<Profile>().await
            .context("Failed to parse profile")?;
        
        Ok(profile)
    }

    async fn get_recovery(&self, token: &str) -> Result<Vec<Recovery>> {
        let end = chrono::Utc::now();
        let start = end - chrono::Duration::days(7);
        
        let url = format!(
            "{}/v2/recovery?start={}&end={}",
            API_BASE,
            start.to_rfc3339(),
            end.to_rfc3339()
        );
        
        let response = self.client
            .get(&url)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .send()
            .await?;

        let json: Value = response.json().await?;
        let records = json["records"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        
        let recovery: Vec<Recovery> = serde_json::from_value(serde_json::Value::Array(records))
            .context("Failed to parse recovery")?;
        
        Ok(recovery)
    }

    async fn get_sleep(&self, token: &str) -> Result<Vec<Sleep>> {
        let end = chrono::Utc::now();
        let start = end - chrono::Duration::days(7);
        
        let url = format!(
            "{}/v2/activity/sleep?start={}&end={}",
            API_BASE,
            start.to_rfc3339(),
            end.to_rfc3339()
        );
        
        let response = self.client
            .get(&url)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .send()
            .await?;

        let json: Value = response.json().await?;
        let records = json["records"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        
        let sleep: Vec<Sleep> = serde_json::from_value(serde_json::Value::Array(records))
            .context("Failed to parse sleep")?;
        
        Ok(sleep)
    }

    async fn get_workouts(&self, token: &str) -> Result<Vec<Workout>> {
        let end = chrono::Utc::now();
        let start = end - chrono::Duration::days(7);
        
        let url = format!(
            "{}/v2/activity/workout?start={}&end={}",
            API_BASE,
            start.to_rfc3339(),
            end.to_rfc3339()
        );
        
        let response = self.client
            .get(&url)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .send()
            .await?;

        let json: Value = response.json().await?;
        let records = json["records"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        
        let workouts: Vec<Workout> = serde_json::from_value(serde_json::Value::Array(records))
            .context("Failed to parse workouts")?;
        
        Ok(workouts)
    }
}
