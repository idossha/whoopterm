use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use oauth2::{AuthorizationCode, AuthUrl, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope, TokenResponse, TokenUrl};
use oauth2::basic::BasicClient;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

use crate::config::Config;

const AUTH_URL: &str = "https://api.prod.whoop.com/oauth/oauth2/auth";
const TOKEN_URL: &str = "https://api.prod.whoop.com/oauth/oauth2/token";
const REDIRECT_URI: &str = "http://localhost:8080/callback";

pub struct AuthManager {
    config: Config,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

impl AuthManager {
    pub fn new() -> Self {
        Self {
            config: Config::load(),
        }
    }

    pub async fn authenticate(&self) -> Result<()> {
        let client_id = self.config.client_id.as_ref()
            .context("Client ID not configured")?;
        let client_secret = self.config.client_secret.as_ref()
            .context("Client secret not configured")?;

        let client = BasicClient::new(
            ClientId::new(client_id.clone()),
            Some(ClientSecret::new(client_secret.clone())),
            AuthUrl::new(AUTH_URL.to_string())?,
            Some(TokenUrl::new(TOKEN_URL.to_string())?),
        )
        .set_redirect_uri(RedirectUrl::new(REDIRECT_URI.to_string())?);

        let (auth_url, _csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("read:recovery".to_string()))
            .add_scope(Scope::new("read:sleep".to_string()))
            .add_scope(Scope::new("read:workout".to_string()))
            .add_scope(Scope::new("read:cycles".to_string()))
            .add_scope(Scope::new("read:profile".to_string()))
            .add_scope(Scope::new("offline".to_string()))
            .url();

        // Open browser
        #[cfg(target_os = "macos")]
        std::process::Command::new("open").arg(auth_url.as_str()).spawn()?;
        
        #[cfg(target_os = "linux")]
        std::process::Command::new("xdg-open").arg(auth_url.as_str()).spawn()?;
        
        println!("Opening browser for WHOOP authentication...");
        println!("If the browser doesn't open, visit: {}", auth_url);

        // Start local server to receive callback
        let code = self.receive_auth_code().await?;

        // Exchange code for token
        let token = client
            .exchange_code(AuthorizationCode::new(code))
            .request_async(oauth2::reqwest::async_http_client)
            .await
            .context("Failed to exchange code for token")?;

        let tokens = Tokens {
            access_token: token.access_token().secret().clone(),
            refresh_token: token.refresh_token().map(|t| t.secret().clone()),
            expires_at: chrono::Utc::now() + chrono::Duration::seconds(token.expires_in().unwrap_or_default().as_secs() as i64),
        };

        self.config.save_tokens(&tokens)?;
        println!("Authentication successful!");

        Ok(())
    }

    pub async fn get_access_token(&self) -> Result<String> {
        if let Ok(tokens) = self.config.load_tokens() {
            if chrono::Utc::now() < tokens.expires_at - chrono::Duration::minutes(5) {
                return Ok(tokens.access_token);
            }

            // Token expired, try to refresh
            if let Some(refresh_token) = &tokens.refresh_token {
                if let Ok(new_tokens) = self.refresh_token(refresh_token).await {
                    self.config.save_tokens(&new_tokens)?;
                    return Ok(new_tokens.access_token);
                }
            }
        }

        Err(anyhow::anyhow!("Not authenticated. Run: whoop --auth"))
    }

    async fn refresh_token(&self, refresh_token: &str) -> Result<Tokens> {
        let client_id = self.config.client_id.as_ref()
            .context("Client ID not configured")?;
        let client_secret = self.config.client_secret.as_ref()
            .context("Client secret not configured")?;

        let client = BasicClient::new(
            ClientId::new(client_id.clone()),
            Some(ClientSecret::new(client_secret.clone())),
            AuthUrl::new(AUTH_URL.to_string())?,
            Some(TokenUrl::new(TOKEN_URL.to_string())?),
        );

        let token = client
            .exchange_refresh_token(&oauth2::RefreshToken::new(refresh_token.to_string()))
            .request_async(oauth2::reqwest::async_http_client)
            .await
            .context("Failed to refresh token")?;

        Ok(Tokens {
            access_token: token.access_token().secret().clone(),
            refresh_token: token.refresh_token().map(|t| t.secret().clone()),
            expires_at: chrono::Utc::now() + chrono::Duration::seconds(token.expires_in().unwrap_or_default().as_secs() as i64),
        })
    }

    async fn receive_auth_code(&self) -> Result<String> {
        let listener = TcpListener::bind("127.0.0.1:8080")
            .context("Failed to bind to port 8080")?;
        
        println!("Waiting for authentication...");

        for stream in listener.incoming() {
            let stream = stream?;
            let mut reader = BufReader::new(&stream);
            let mut line = String::new();
            
            reader.read_line(&mut line)?;
            
            // Parse request line
            if line.starts_with("GET /callback") {
                // Extract code from URL
                if let Some(code_start) = line.find("code=") {
                    let code = line[code_start + 5..]
                        .split_whitespace()
                        .next()
                        .context("Failed to parse auth code")?;
                    
                    // Send response
                    let response = "HTTP/1.1 200 OK\r\nContent-Length: 32\r\n\r\nAuthentication successful!";
                    let mut writer = &stream;
                    writer.write_all(response.as_bytes())?;
                    
                    return Ok(code.to_string());
                }
            }
        }

        Err(anyhow::anyhow!("Failed to receive auth code"))
    }
}
