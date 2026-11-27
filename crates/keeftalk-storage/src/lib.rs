//! Cloudflare R2 presigned object storage client via the `storage` Edge Function.

use keeftalk_api::{ApiError, SupabaseClient};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("cloud storage quota exceeded (prepaid balance empty after 1 GB free allowance)")]
    PrepaidBalanceEmpty,

    #[error("regional storage bucket unavailable: {0}")]
    InfrastructureUnavailable(String),

    #[error("storage API error: {0}")]
    Api(#[from] ApiError),

    #[error("HTTP transfer error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("storage transfer failed ({status})")]
    Transfer { status: u16 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadTarget {
    pub url: String,
    pub object_key: String,
    pub bucket: String,
    pub country_code: String,
    pub region: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadTarget {
    pub url: String,
    pub object_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalizeResponse {
    pub success: bool,
    #[serde(default)]
    pub size: u64,
    #[serde(default)]
    pub error: Option<String>,
}

pub struct R2StorageClient {
    api: SupabaseClient,
    http: reqwest::Client,
}

impl R2StorageClient {
    pub fn new(api: SupabaseClient) -> Self {
        Self {
            api,
            http: reqwest::Client::new(),
        }
    }

    pub async fn get_upload_url(
        &self,
        file_id: &str,
        category: &str,
        subpath: &str,
        access_token: &str,
    ) -> Result<UploadTarget, StorageError> {
        let body = serde_json::json!({
            "action": "get-upload-url",
            "fileId": file_id,
            "category": category,
            "subpath": subpath,
        });
        self.api
            .invoke_function("storage", &body, access_token)
            .await
            .map_err(map_api_err)
    }

    pub async fn put_encrypted_bytes(
        &self,
        presigned_url: &str,
        ciphertext: Vec<u8>,
    ) -> Result<(), StorageError> {
        let res = self.http.put(presigned_url).body(ciphertext).send().await?;
        let status = res.status().as_u16();
        if (200..300).contains(&status) {
            Ok(())
        } else {
            Err(StorageError::Transfer { status })
        }
    }

    pub async fn finalize_upload(
        &self,
        file_id: &str,
        object_key: &str,
        access_token: &str,
    ) -> Result<u64, StorageError> {
        let body = serde_json::json!({
            "action": "finalize-upload",
            "fileId": file_id,
            "path": object_key,
        });
        let resp: FinalizeResponse = self
            .api
            .invoke_function("storage", &body, access_token)
            .await
            .map_err(map_api_err)?;
        if resp.success {
            Ok(resp.size)
        } else {
            Err(StorageError::Transfer { status: 500 })
        }
    }

    pub async fn get_download_url(
        &self,
        file_id: &str,
        object_key: Option<&str>,
        access_token: &str,
    ) -> Result<DownloadTarget, StorageError> {
        let body = serde_json::json!({
            "action": "get-download-url",
            "fileId": file_id,
            "path": object_key,
        });
        self.api
            .invoke_function("storage", &body, access_token)
            .await
            .map_err(map_api_err)
    }
}

fn map_api_err(err: ApiError) -> StorageError {
    match err {
        ApiError::PaymentRequired(_) => StorageError::PrepaidBalanceEmpty,
        ApiError::InfrastructureUnavailable(msg) => StorageError::InfrastructureUnavailable(msg),
        other => StorageError::Api(other),
    }
}
