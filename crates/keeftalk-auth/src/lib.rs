//! Supabase GoTrue authentication, secure session storage, and AEK recovery/initialization.

use std::sync::{Arc, RwLock};

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use keeftalk_api::{ApiError, SupabaseClient};
use keeftalk_crypto::{
    AccountKeyContext, CryptoError, EncryptedObject, derive_kek, generate_salt,
};
use keeftalk_platform::{PlatformError, SecretStore};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{info, warn};
use zeroize::{Zeroize, ZeroizeOnDrop};

const SESSION_SECRET_KEY: &str = "supabase_session_v1";

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("authentication failed: {0}")]
    Failed(String),

    #[error("security recovery required to unlock Account Encryption Key")]
    RecoveryRequired,

    #[error("API error: {0}")]
    Api(#[from] ApiError),

    #[error("cryptographic error: {0}")]
    Crypto(#[from] CryptoError),

    #[error("secure storage error: {0}")]
    Platform(#[from] PlatformError),

    #[error("serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthState {
    LocalOffline,
    MockDemo {
        user_id: String,
        display_name: String,
    },
    SignedOut,
    SignedIn {
        user_id: String,
        email: Option<String>,
    },
    RecoveryRequired {
        user_id: String,
    },
}

#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct SessionTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub user_id: String,
    pub email: Option<String>,
    pub expires_at_unix: i64,
}

impl std::fmt::Debug for SessionTokens {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionTokens")
            .field("user_id", &self.user_id)
            .field("email", &self.email)
            .field("access_token", &"[REDACTED]")
            .field("refresh_token", &"[REDACTED]")
            .finish()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserSecuritySettingsDto {
    pub user_id: String,
    #[serde(default)]
    pub encrypted_account_key: Option<String>,
    #[serde(default)]
    pub key_salt: Option<String>,
    #[serde(default)]
    pub key_nonce: Option<String>,
    #[serde(default)]
    pub verification_tag: Option<String>,
}

pub struct AuthManager {
    api: SupabaseClient,
    secret_store: Arc<dyn SecretStore>,
    session: RwLock<Option<SessionTokens>>,
    aek: RwLock<Option<AccountKeyContext>>,
}

impl AuthManager {
    pub fn new(api: SupabaseClient, secret_store: Arc<dyn SecretStore>) -> Self {
        Self {
            api,
            secret_store,
            session: RwLock::new(None),
            aek: RwLock::new(None),
        }
    }

    pub fn restore_persisted_session(&self) -> Result<AuthState, AuthError> {
        let Some(raw) = self.secret_store.get_secret(SESSION_SECRET_KEY)? else {
            return Ok(AuthState::SignedOut);
        };
        let tokens: SessionTokens = serde_json::from_str(&raw)?;
        let user_id = tokens.user_id.clone();
        *self.session.write().map_err(|_| AuthError::Failed("lock".into()))? = Some(tokens);
        Ok(AuthState::RecoveryRequired { user_id })
    }

    pub async fn sign_in(&self, email: &str, password: &str) -> Result<AuthState, AuthError> {
        let resp = self.api.auth_sign_in_password(email, password).await?;
        let now_unix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let tokens = SessionTokens {
            access_token: resp.access_token.clone(),
            refresh_token: resp.refresh_token,
            user_id: resp.user.id.clone(),
            email: resp.user.email.clone(),
            expires_at_unix: now_unix + resp.expires_in,
        };

        let serialized = zeroize::Zeroizing::new(serde_json::to_string(&tokens)?);
        self.secret_store
            .set_secret(SESSION_SECRET_KEY, &serialized)?;
        *self.session.write().map_err(|_| AuthError::Failed("lock".into()))? = Some(tokens);

        self.recover_or_init_aek(&resp.user.id, password, &resp.access_token)
            .await?;

        Ok(AuthState::SignedIn {
            user_id: resp.user.id,
            email: resp.user.email,
        })
    }

    pub async fn recover_aek(&self, password: &str) -> Result<AuthState, AuthError> {
        let (user_id, email, access_token) = {
            let guard = self.session.read().map_err(|_| AuthError::Failed("lock".into()))?;
            let s = guard.as_ref().ok_or_else(|| AuthError::Failed("not signed in".into()))?;
            (s.user_id.clone(), s.email.clone(), s.access_token.clone())
        };

        self.recover_or_init_aek(&user_id, password, &access_token)
            .await?;

        Ok(AuthState::SignedIn { user_id, email })
    }

    async fn recover_or_init_aek(
        &self,
        user_id: &str,
        password: &str,
        access_token: &str,
    ) -> Result<(), AuthError> {
        let filter = format!("eq.{user_id}");
        let rows: Vec<UserSecuritySettingsDto> = self
            .api
            .rest_select(
                "user_security_settings",
                &[("user_id", filter.as_str()), ("select", "*")],
                access_token,
            )
            .await?;

        if let Some(settings) = rows.into_iter().next()
            && let (Some(enc_aek), Some(salt_b64), Some(nonce_b64)) = (
                settings.encrypted_account_key,
                settings.key_salt,
                settings.key_nonce,
            )
        {
            let salt = STANDARD
                .decode(salt_b64)
                .map_err(|e| CryptoError::InvalidFormat(e.to_string()))?;
            let kek = derive_kek(password, &salt)?;
            let wrapped = EncryptedObject {
                version: 1,
                key_id: "kek".to_string(),
                iv: nonce_b64,
                ciphertext: enc_aek,
                tag: None,
            };
            let ctx = AccountKeyContext::decrypt_aek_with_kek(&wrapped, &kek)?;
            if let Some(tag_json) = settings.verification_tag {
                let tag: EncryptedObject = serde_json::from_str(&tag_json)?;
                ctx.validate_verification_tag(&tag)?;
            }
            *self.aek.write().map_err(|_| AuthError::Failed("lock".into()))? = Some(ctx);
            info!(user_id, "Recovered Account Encryption Key from cloud");
            return Ok(());
        }

        // Initialize new AEK if none exists on server
        let ctx = AccountKeyContext::generate();
        let salt = generate_salt();
        let kek = derive_kek(password, &salt)?;
        let wrapped = ctx.encrypt_aek_with_kek(&kek)?;
        let tag = ctx.generate_verification_tag()?;

        let payload = serde_json::json!({
            "user_id": user_id,
            "encrypted_account_key": wrapped.ciphertext,
            "key_salt": STANDARD.encode(salt),
            "key_nonce": wrapped.iv,
            "verification_tag": serde_json::to_string(&tag)?,
        });
        let _: Vec<serde_json::Value> = self
            .api
            .rest_upsert("user_security_settings", &payload, access_token)
            .await?;

        *self.aek.write().map_err(|_| AuthError::Failed("lock".into()))? = Some(ctx);
        info!(user_id, "Initialized and uploaded new Account Encryption Key");
        Ok(())
    }

    pub fn account_key_context(&self) -> Result<AccountKeyContext, AuthError> {
        self.aek
            .read()
            .map_err(|_| AuthError::Failed("lock".into()))?
            .clone()
            .ok_or(AuthError::RecoveryRequired)
    }

    pub async fn sign_out(&self) -> Result<AuthState, AuthError> {
        let token_opt = {
            let mut guard = self.session.write().map_err(|_| AuthError::Failed("lock".into()))?;
            guard.take().map(|s| s.access_token.clone())
        };
        if let Some(token) = token_opt
            && let Err(e) = self.api.auth_sign_out(&token).await
        {
            warn!("Remote logout returned non-fatal error: {e}");
        }
        *self.aek.write().map_err(|_| AuthError::Failed("lock".into()))? = None;
        self.secret_store.delete_secret(SESSION_SECRET_KEY)?;
        Ok(AuthState::SignedOut)
    }
}
