//! Supabase PostgREST, GoTrue Auth, Edge Functions, and Realtime WebSocket client.

pub mod realtime;

use std::time::Duration;

use keeftalk_settings::BackendSection;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use url::Url;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("backend endpoint is not configured")]
    NotConfigured,

    #[error("invalid backend URL: {0}")]
    InvalidUrl(#[from] url::ParseError),

    #[error("network transport error: {0}")]
    Transport(#[from] reqwest::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("prepaid cloud balance exhausted (HTTP 402): {0}")]
    PaymentRequired(String),

    #[error("backend infrastructure unavailable (HTTP 503): {0}")]
    InfrastructureUnavailable(String),

    #[error("HTTP {status}: {message}")]
    Http { status: u16, message: String },

    #[error("realtime connection error: {0}")]
    Realtime(String),
}

#[derive(Debug, Clone)]
pub struct BackendEndpoints {
    pub base_url: Url,
    pub anon_key: String,
    pub timeout: Duration,
}

impl BackendEndpoints {
    pub fn from_settings(section: &BackendSection) -> Result<Self, ApiError> {
        if section.supabase_url.trim().is_empty() || section.supabase_anon_key.trim().is_empty() {
            return Err(ApiError::NotConfigured);
        }
        Ok(Self {
            base_url: Url::parse(section.supabase_url.trim())?,
            anon_key: section.supabase_anon_key.trim().to_string(),
            timeout: Duration::from_secs(section.request_timeout_secs.max(5)),
        })
    }

    pub fn from_raw(base_url: &str, anon_key: &str) -> Result<Self, ApiError> {
        Ok(Self {
            base_url: Url::parse(base_url.trim())?,
            anon_key: anon_key.trim().to_string(),
            timeout: Duration::from_secs(30),
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GoTrueUser {
    pub id: String,
    #[serde(default)]
    pub email: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GoTrueSessionResponse {
    pub access_token: String,
    pub refresh_token: String,
    #[serde(default)]
    pub expires_in: i64,
    #[serde(default)]
    pub token_type: Option<String>,
    pub user: GoTrueUser,
}

#[derive(Clone)]
pub struct SupabaseClient {
    endpoints: BackendEndpoints,
    http: Client,
}

impl SupabaseClient {
    pub fn new(endpoints: BackendEndpoints) -> Result<Self, ApiError> {
        let http = Client::builder().timeout(endpoints.timeout).build()?;
        Ok(Self { endpoints, http })
    }

    pub fn endpoints(&self) -> &BackendEndpoints {
        &self.endpoints
    }

    fn endpoint_url(&self, path: &str) -> Result<Url, ApiError> {
        self.endpoints
            .base_url
            .join(path.trim_start_matches('/'))
            .map_err(ApiError::InvalidUrl)
    }

    pub async fn auth_sign_in_password(
        &self,
        email: &str,
        password: &str,
    ) -> Result<GoTrueSessionResponse, ApiError> {
        let url = self.endpoint_url("auth/v1/token?grant_type=password")?;
        let res = self
            .http
            .post(url)
            .header("apikey", &self.endpoints.anon_key)
            .json(&serde_json::json!({
                "email": email,
                "password": password,
            }))
            .send()
            .await?;
        self.decode_response(res).await
    }

    pub async fn auth_sign_up(
        &self,
        email: &str,
        password: &str,
    ) -> Result<Value, ApiError> {
        let url = self.endpoint_url("auth/v1/signup")?;
        let res = self
            .http
            .post(url)
            .header("apikey", &self.endpoints.anon_key)
            .json(&serde_json::json!({
                "email": email,
                "password": password,
            }))
            .send()
            .await?;
        self.decode_response(res).await
    }

    pub async fn auth_refresh_token(
        &self,
        refresh_token: &str,
    ) -> Result<GoTrueSessionResponse, ApiError> {
        let url = self.endpoint_url("auth/v1/token?grant_type=refresh_token")?;
        let res = self
            .http
            .post(url)
            .header("apikey", &self.endpoints.anon_key)
            .json(&serde_json::json!({
                "refresh_token": refresh_token,
            }))
            .send()
            .await?;
        self.decode_response(res).await
    }

    pub async fn auth_sign_out(&self, access_token: &str) -> Result<(), ApiError> {
        let url = self.endpoint_url("auth/v1/logout")?;
        let res = self
            .http
            .post(url)
            .header("apikey", &self.endpoints.anon_key)
            .bearer_auth(access_token)
            .send()
            .await?;
        if res.status().is_success() {
            Ok(())
        } else {
            let status = res.status().as_u16();
            let message = res.text().await.unwrap_or_default();
            Err(ApiError::Http { status, message })
        }
    }

    pub async fn rest_select<T: DeserializeOwned>(
        &self,
        table: &str,
        query: &[(&str, &str)],
        access_token: &str,
    ) -> Result<Vec<T>, ApiError> {
        let url = self.endpoint_url(&format!("rest/v1/{table}"))?;
        let res = self
            .http
            .get(url)
            .query(query)
            .header("apikey", &self.endpoints.anon_key)
            .bearer_auth(access_token)
            .send()
            .await?;
        self.decode_response(res).await
    }

    pub async fn rest_upsert<B: Serialize, T: DeserializeOwned>(
        &self,
        table: &str,
        body: &B,
        access_token: &str,
    ) -> Result<Vec<T>, ApiError> {
        let url = self.endpoint_url(&format!("rest/v1/{table}"))?;
        let res = self
            .http
            .post(url)
            .header("apikey", &self.endpoints.anon_key)
            .bearer_auth(access_token)
            .header("Prefer", "resolution=merge-duplicates,return=representation")
            .json(body)
            .send()
            .await?;
        self.decode_response(res).await
    }

    pub async fn invoke_function<B: Serialize, T: DeserializeOwned>(
        &self,
        function_name: &str,
        body: &B,
        access_token: &str,
    ) -> Result<T, ApiError> {
        let url = self.endpoint_url(&format!("functions/v1/{function_name}"))?;
        let res = self
            .http
            .post(url)
            .header("apikey", &self.endpoints.anon_key)
            .bearer_auth(access_token)
            .json(body)
            .send()
            .await?;
        self.decode_response(res).await
    }

    async fn decode_response<T: DeserializeOwned>(
        &self,
        res: reqwest::Response,
    ) -> Result<T, ApiError> {
        let status = res.status().as_u16();
        let text = res.text().await?;

        if (200..300).contains(&status) {
            return serde_json::from_str(&text).map_err(ApiError::Json);
        }

        if status == 402 || text.contains("PREPAID_BALANCE_EMPTY") {
            return Err(ApiError::PaymentRequired(text));
        }
        if status == 503 || text.contains("INFRASTRUCTURE_ERROR") {
            return Err(ApiError::InfrastructureUnavailable(text));
        }

        Err(ApiError::Http {
            status,
            message: text,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    async fn spawn_one_shot_http(status_line: &'static str, body: &'static str) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("addr");
        tokio::spawn(async move {
            if let Ok((mut socket, _)) = listener.accept().await {
                let mut buf = [0u8; 2048];
                let _ = socket.read(&mut buf).await;
                let resp = format!(
                    "HTTP/1.1 {status_line}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = socket.write_all(resp.as_bytes()).await;
            }
        });
        format!("http://{addr}")
    }

    #[tokio::test]
    async fn sign_in_and_402_handling_work_against_mock_server() {
        let auth_url = spawn_one_shot_http(
            "200 OK",
            r#"{"access_token":"tok","refresh_token":"ref","expires_in":3600,"user":{"id":"u1","email":"a@b.c"}}"#,
        )
        .await;
        let client =
            SupabaseClient::new(BackendEndpoints::from_raw(&auth_url, "anon").expect("ep"))
                .expect("client");
        let sess = client
            .auth_sign_in_password("a@b.c", "secret")
            .await
            .expect("sign in");
        assert_eq!(sess.user.id, "u1");

        let quota_url = spawn_one_shot_http(
            "402 Payment Required",
            r#"{"code":"PREPAID_BALANCE_EMPTY","error":"1 GB allowance reached"}"#,
        )
        .await;
        let client2 =
            SupabaseClient::new(BackendEndpoints::from_raw(&quota_url, "anon").expect("ep"))
                .expect("client2");
        let err = client2
            .invoke_function::<Value, Value>("storage", &serde_json::json!({}), "tok")
            .await
            .expect_err("must return 402 error");
        assert!(matches!(err, ApiError::PaymentRequired(_)));
    }
}
