use anyhow::Result;
use reqwest::header::AUTHORIZATION;
use serde_json::Value;
use std::time::Duration;

use crate::config::Config;
use crate::data::{DashboardData, Profile, Recovery, Sleep, Workout};
use crate::auth::AuthManager;

fn url_encode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            ':' => "%3A".to_string(),
            '+' => "%2B".to_string(),
            ' ' => "%20".to_string(),
            c => c.to_string(),
        })
        .collect()
}

const API_BASE: &str = "https://api.prod.whoop.com/developer";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("API request failed: {endpoint} returned {status} - {message}")]
    RequestFailed {
        endpoint: String,
        status: u16,
        message: String,
    },
    #[error("Failed to parse response from {endpoint}: {source}")]
    ParseError {
        endpoint: String,
        #[source]
        source: anyhow::Error,
    },
}

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

        let status = response.status();
        if status.is_success() {
            Ok(())
        } else {
            let body = response.text().await.unwrap_or_default();
            let message = body.chars().take(200).collect::<String>();
            Err(ApiError::RequestFailed {
                endpoint: "/v2/user/profile/basic".to_string(),
                status: status.as_u16(),
                message,
            }.into())
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

    async fn check_response(&self, response: reqwest::Response, endpoint: &str) -> Result<String, ApiError> {
        let status = response.status();
        let endpoint_str = endpoint.to_string();
        
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            let message = if body.is_empty() {
                "Empty response".to_string()
            } else {
                body.chars().take(500).collect::<String>()
            };
            return Err(ApiError::RequestFailed {
                endpoint: endpoint_str,
                status: status.as_u16(),
                message,
            });
        }
        
        response.text().await.map_err(|e| ApiError::RequestFailed {
            endpoint: endpoint_str.clone(),
            status: status.as_u16(),
            message: format!("Failed to read response body: {}", e),
        })
    }

    async fn get_profile(&self, token: &str) -> Result<Profile> {
        let url = format!("{}/v2/user/profile/basic", API_BASE);
        let endpoint = "/v2/user/profile/basic";
        
        let response = self.client
            .get(&url)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .send()
            .await?;

        let body = self.check_response(response, endpoint).await?;
        let profile = serde_json::from_str::<Profile>(&body)
            .map_err(|e| ApiError::ParseError {
                endpoint: endpoint.to_string(),
                source: anyhow::anyhow!("{} (body excerpt: {})", e, &body[..body.len().min(200)]),
            })?;
        
        Ok(profile)
    }

    async fn get_recovery(&self, token: &str) -> Result<Vec<Recovery>> {
        let end = chrono::Utc::now();
        let start = end - chrono::Duration::days(7);
        
        let url = format!(
            "{}/v2/recovery?start={}&end={}",
            API_BASE,
            url_encode(&start.to_rfc3339()),
            url_encode(&end.to_rfc3339())
        );
        let endpoint = "/v2/recovery";
        
        let response = self.client
            .get(&url)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .send()
            .await?;

        let body = self.check_response(response, endpoint).await?;
        let json: Value = serde_json::from_str(&body)
            .map_err(|e| ApiError::ParseError {
                endpoint: endpoint.to_string(),
                source: anyhow::anyhow!("Failed to parse JSON: {} (body excerpt: {})", e, &body[..body.len().min(200)]),
            })?;
        
        let records = json["records"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        
        let recovery: Vec<Recovery> = serde_json::from_value(serde_json::Value::Array(records))
            .map_err(|e| ApiError::ParseError {
                endpoint: endpoint.to_string(),
                source: anyhow::anyhow!("Failed to parse recovery records: {}", e),
            })?;
        
        // Filter out unscored records for cleaner display
        let scored: Vec<Recovery> = recovery.into_iter()
            .filter(|r| r.score.is_some())
            .collect();
        
        Ok(scored)
    }

    async fn get_sleep(&self, token: &str) -> Result<Vec<Sleep>> {
        let end = chrono::Utc::now();
        let start = end - chrono::Duration::days(7);
        
        let url = format!(
            "{}/v2/activity/sleep?start={}&end={}",
            API_BASE,
            url_encode(&start.to_rfc3339()),
            url_encode(&end.to_rfc3339())
        );
        let endpoint = "/v2/activity/sleep";
        
        let response = self.client
            .get(&url)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .send()
            .await?;

        let body = self.check_response(response, endpoint).await?;
        let json: Value = serde_json::from_str(&body)
            .map_err(|e| ApiError::ParseError {
                endpoint: endpoint.to_string(),
                source: anyhow::anyhow!("Failed to parse JSON: {} (body excerpt: {})", e, &body[..body.len().min(200)]),
            })?;
        
        let records = json["records"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        
        let sleep: Vec<Sleep> = serde_json::from_value(serde_json::Value::Array(records))
            .map_err(|e| ApiError::ParseError {
                endpoint: endpoint.to_string(),
                source: anyhow::anyhow!("Failed to parse sleep records: {}", e),
            })?;
        
        // Filter out unscored and nap records for main display
        let scored_nights: Vec<Sleep> = sleep.into_iter()
            .filter(|s| s.score.is_some() && !s.nap)
            .collect();
        
        Ok(scored_nights)
    }

    async fn get_workouts(&self, token: &str) -> Result<Vec<Workout>> {
        let end = chrono::Utc::now();
        let start = end - chrono::Duration::days(7);
        
        let url = format!(
            "{}/v2/activity/workout?start={}&end={}",
            API_BASE,
            url_encode(&start.to_rfc3339()),
            url_encode(&end.to_rfc3339())
        );
        let endpoint = "/v2/activity/workout";
        
        let response = self.client
            .get(&url)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .send()
            .await?;

        let body = self.check_response(response, endpoint).await?;
        let json: Value = serde_json::from_str(&body)
            .map_err(|e| ApiError::ParseError {
                endpoint: endpoint.to_string(),
                source: anyhow::anyhow!("Failed to parse JSON: {} (body excerpt: {})", e, &body[..body.len().min(200)]),
            })?;
        
        let records = json["records"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        
        let workouts: Vec<Workout> = serde_json::from_value(serde_json::Value::Array(records))
            .map_err(|e| ApiError::ParseError {
                endpoint: endpoint.to_string(),
                source: anyhow::anyhow!("Failed to parse workout records: {}", e),
            })?;
        
        // Filter out unscored records
        let scored: Vec<Workout> = workouts.into_iter()
            .filter(|w| w.score.is_some())
            .collect();
        
        Ok(scored)
    }
}
