use std::{
    collections::HashSet,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use hmac::{Hmac, Mac};
use rand::RngCore;
use secrecy::{ExposeSecret, SecretString};
use sha2::{Digest, Sha256};
use tokio::sync::RwLock;

use crate::error::{AppError, AppResult};

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone)]
pub struct WrappedApiKeyService {
    secret: SecretString,
}

impl WrappedApiKeyService {
    pub fn new(secret: SecretString) -> Self {
        Self { secret }
    }

    pub fn issue(&self, client_id: &str, ttl: Duration) -> AppResult<String> {
        if client_id.trim().is_empty() {
            return Err(AppError::Validation(
                "client_id must not be empty".to_string(),
            ));
        }

        let expiry = now_epoch() + ttl.as_secs();
        let payload = format!("{client_id}:{expiry}");
        let signature = self.sign(&payload)?;
        Ok(format!("mcp_prism_v1:{client_id}:{signature}:{expiry}"))
    }

    pub fn validate(&self, token: &str) -> AppResult<WrappedApiKeyClaims> {
        let parts: Vec<&str> = token.split(':').collect();
        if parts.len() != 4 {
            return Err(AppError::Auth("invalid wrapped key format".to_string()));
        }

        if parts[0] != "mcp_prism_v1" {
            return Err(AppError::Auth(
                "unsupported wrapped key version".to_string(),
            ));
        }

        let expiry = parts[3]
            .parse::<u64>()
            .map_err(|_| AppError::Auth("invalid wrapped key expiry".to_string()))?;
        if expiry <= now_epoch() {
            return Err(AppError::Auth("wrapped key expired".to_string()));
        }

        let payload = format!("{}:{}", parts[1], parts[3]);
        let expected = self.sign(&payload)?;
        if expected != parts[2] {
            return Err(AppError::Auth("wrapped key signature mismatch".to_string()));
        }

        Ok(WrappedApiKeyClaims {
            client_id: parts[1].to_string(),
            signature: parts[2].to_string(),
            expiry,
        })
    }

    fn sign(&self, payload: &str) -> AppResult<String> {
        let mut mac = <HmacSha256 as Mac>::new_from_slice(self.secret.expose_secret().as_bytes())
            .map_err(|err| AppError::Auth(err.to_string()))?;
        mac.update(payload.as_bytes());
        Ok(hex::encode(mac.finalize().into_bytes()))
    }
}

#[derive(Debug, Clone)]
pub struct WrappedApiKeyClaims {
    pub client_id: String,
    pub signature: String,
    pub expiry: u64,
}

#[derive(Default)]
pub struct RevocationRegistry {
    client_ids: RwLock<HashSet<String>>,
    signatures: RwLock<HashSet<String>>,
}

impl RevocationRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn revoke_client(&self, client_id: &str) {
        self.client_ids.write().await.insert(client_id.to_string());
    }

    pub async fn revoke_token_signature(&self, signature: &str) {
        self.signatures.write().await.insert(signature.to_string());
    }

    pub async fn is_revoked(&self, claims: &WrappedApiKeyClaims) -> bool {
        self.client_ids.read().await.contains(&claims.client_id)
            || self.signatures.read().await.contains(&claims.signature)
    }
}

#[derive(Debug, Clone)]
pub struct SecretCipher {
    key: [u8; 32],
}

impl SecretCipher {
    pub fn new(secret: &SecretString) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(secret.expose_secret().as_bytes());
        let digest = hasher.finalize();
        let mut key = [0_u8; 32];
        key.copy_from_slice(&digest[..32]);
        Self { key }
    }

    pub fn encrypt(&self, plaintext: &str) -> AppResult<String> {
        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|err| AppError::Internal(err.to_string()))?;
        let mut nonce_bytes = [0_u8; 12];
        rand::rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|err| AppError::Internal(err.to_string()))?;

        Ok(format!(
            "{}.{}",
            STANDARD.encode(nonce_bytes),
            STANDARD.encode(ciphertext)
        ))
    }

    pub fn decrypt(&self, blob: &str) -> AppResult<String> {
        let parts: Vec<&str> = blob.split('.').collect();
        if parts.len() != 2 {
            return Err(AppError::Validation(
                "invalid encrypted secret blob".to_string(),
            ));
        }

        let nonce = STANDARD
            .decode(parts[0])
            .map_err(|err| AppError::Validation(err.to_string()))?;
        let ciphertext = STANDARD
            .decode(parts[1])
            .map_err(|err| AppError::Validation(err.to_string()))?;

        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|err| AppError::Internal(err.to_string()))?;
        let plaintext = cipher
            .decrypt(Nonce::from_slice(&nonce), ciphertext.as_ref())
            .map_err(|err| AppError::Auth(err.to_string()))?;

        String::from_utf8(plaintext).map_err(|err| AppError::Internal(err.to_string()))
    }
}

fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrapped_key_round_trip() {
        let service =
            WrappedApiKeyService::new(SecretString::new("secret".to_string().into_boxed_str()));
        let token = service.issue("client-a", Duration::from_secs(60)).unwrap();
        let claims = service.validate(&token).unwrap();
        assert_eq!(claims.client_id, "client-a");
        assert!(!claims.signature.is_empty());
        assert!(claims.expiry > 0);
    }

    #[test]
    fn secret_cipher_round_trip() {
        let cipher = SecretCipher::new(&SecretString::new("enc".to_string().into_boxed_str()));
        let encrypted = cipher.encrypt("hello").unwrap();
        let decrypted = cipher.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, "hello");
    }
}
