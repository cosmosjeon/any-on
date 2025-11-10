use std::{ops::Deref, sync::Arc};

use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use base64::Engine;
use db::{DBService, models::secret::SecretRecord};
use rand::{RngCore, rngs::OsRng};
use thiserror::Error;

const SECRET_KEY_ENV: &str = "ANYON_SECRET_KEY";
const NONCE_LEN: usize = 12;
const DEFAULT_KEY_VERSION: i64 = 1;

/// Descriptor used to read/write a specific secret.
#[derive(Debug, Clone, Copy)]
pub struct SecretDescriptor<'a> {
    pub provider: &'a str,
    pub name: &'a str,
}

pub const PROVIDER_GITHUB: &str = "github";
pub const PROVIDER_CLAUDE: &str = "claude";
pub const SECRET_NAME_OAUTH: &str = "oauth_token";
pub const SECRET_NAME_PAT: &str = "pat";
pub const SECRET_NAME_CLAUDE_ACCESS: &str = "access_token";
pub const SECRET_GITHUB_OAUTH: SecretDescriptor<'static> = SecretDescriptor {
    provider: PROVIDER_GITHUB,
    name: SECRET_NAME_OAUTH,
};
pub const SECRET_GITHUB_PAT: SecretDescriptor<'static> = SecretDescriptor {
    provider: PROVIDER_GITHUB,
    name: SECRET_NAME_PAT,
};
pub const SECRET_CLAUDE_ACCESS: SecretDescriptor<'static> = SecretDescriptor {
    provider: PROVIDER_CLAUDE,
    name: SECRET_NAME_CLAUDE_ACCESS,
};

#[derive(Debug, Error)]
pub enum SecretStoreError {
    #[error("environment variable ANYON_SECRET_KEY not set")]
    MissingKey,
    #[error("ANYON_SECRET_KEY must be base64 encoded 32 bytes: {0}")]
    InvalidKey(String),
    #[error("encryption error")]
    Encrypt,
    #[error("decryption error")]
    Decrypt,
    #[error("stored secret uses unsupported key version {0}")]
    UnsupportedKeyVersion(i64),
    #[error("secret data is not valid UTF-8")]
    InvalidUtf8,
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

#[derive(Clone)]
pub struct SecretStore {
    db: DBService,
    cipher: Arc<Aes256Gcm>,
    key_version: i64,
}

impl SecretStore {
    pub fn new(db: DBService) -> Result<Self, SecretStoreError> {
        let key_b64 = std::env::var(SECRET_KEY_ENV).map_err(|_| SecretStoreError::MissingKey)?;
        let key_bytes = base64::engine::general_purpose::STANDARD
            .decode(key_b64.as_bytes())
            .map_err(|err| SecretStoreError::InvalidKey(err.to_string()))?;
        if key_bytes.len() != 32 {
            return Err(SecretStoreError::InvalidKey(
                "expected 32 decoded bytes".to_string(),
            ));
        }
        let cipher = Aes256Gcm::new_from_slice(&key_bytes)
            .map_err(|err| SecretStoreError::InvalidKey(err.to_string()))?;
        Ok(Self {
            db,
            cipher: Arc::new(cipher),
            key_version: DEFAULT_KEY_VERSION,
        })
    }

    pub fn db(&self) -> &DBService {
        &self.db
    }

    pub async fn put_secret(
        &self,
        user_id: &str,
        descriptor: SecretDescriptor<'_>,
        value: &[u8],
    ) -> Result<(), SecretStoreError> {
        let mut nonce_bytes = [0u8; NONCE_LEN];
        OsRng.fill_bytes(&mut nonce_bytes);
        #[allow(deprecated)]
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = self
            .cipher
            .encrypt(nonce, value)
            .map_err(|_| SecretStoreError::Encrypt)?;

        let mut blob = Vec::with_capacity(NONCE_LEN + ciphertext.len());
        blob.extend_from_slice(&nonce_bytes);
        blob.extend_from_slice(ciphertext.as_slice());

        SecretRecord::upsert(
            &self.db.pool,
            user_id,
            descriptor.provider,
            descriptor.name,
            &blob,
            self.key_version,
        )
        .await?;
        Ok(())
    }

    pub async fn get_secret(
        &self,
        user_id: &str,
        descriptor: SecretDescriptor<'_>,
    ) -> Result<Option<Vec<u8>>, SecretStoreError> {
        let record =
            SecretRecord::get(&self.db.pool, user_id, descriptor.provider, descriptor.name).await?;

        let Some(record) = record else {
            return Ok(None);
        };

        if record.key_version != self.key_version {
            return Err(SecretStoreError::UnsupportedKeyVersion(record.key_version));
        }
        if record.secret_blob.len() <= NONCE_LEN {
            return Err(SecretStoreError::Decrypt);
        }
        let (nonce_bytes, cipher_bytes) = record.secret_blob.split_at(NONCE_LEN);
        #[allow(deprecated)]
        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = self
            .cipher
            .decrypt(nonce, cipher_bytes)
            .map_err(|_| SecretStoreError::Decrypt)?;

        Ok(Some(plaintext))
    }

    pub async fn get_secret_string(
        &self,
        user_id: &str,
        descriptor: SecretDescriptor<'_>,
    ) -> Result<Option<String>, SecretStoreError> {
        let data = self.get_secret(user_id, descriptor).await?;
        if let Some(bytes) = data {
            let string = String::from_utf8(bytes).map_err(|_| SecretStoreError::InvalidUtf8)?;
            Ok(Some(string))
        } else {
            Ok(None)
        }
    }

    pub async fn delete_secret(
        &self,
        user_id: &str,
        descriptor: SecretDescriptor<'_>,
    ) -> Result<(), SecretStoreError> {
        SecretRecord::delete(&self.db.pool, user_id, descriptor.provider, descriptor.name).await?;
        Ok(())
    }
}

impl Deref for SecretStore {
    type Target = DBService;

    fn deref(&self) -> &Self::Target {
        &self.db
    }
}
