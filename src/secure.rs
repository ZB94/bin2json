use rsa::{Hash, PaddingScheme, PublicKey, RsaPrivateKey, RsaPublicKey};
use rsa::pkcs1::{FromRsaPrivateKey, FromRsaPublicKey};
use sha2::Digest;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "format")]
pub enum SecureKey {
    RsaPkcs1Pem {
        /// 是否为私钥
        secure_key: bool,
        /// 私钥/公钥
        key: String,
        /// 签名/验证时的摘要方式
        #[serde(default)]
        hasher: Hasher,
    },
}

impl SecureKey {
    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, SecureError> {
        match self {
            Self::RsaPkcs1Pem { secure_key: false, key, .. } => {
                RsaPublicKey::from_pkcs1_pem(key)
                    .map_err(|e| SecureError::ParseKeyError(e.to_string()))?
                    .encrypt(&mut rand::rngs::OsRng, PaddingScheme::PKCS1v15Encrypt, data)
                    .map_err(|e| SecureError::EncryptError(e.to_string()))
            }
            _ => Err(SecureError::Unsupported)
        }
    }

    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, SecureError> {
        match self {
            Self::RsaPkcs1Pem { secure_key: true, key, .. } => {
                RsaPrivateKey::from_pkcs1_pem(key)
                    .map_err(|e| SecureError::ParseKeyError(e.to_string()))?
                    .decrypt(PaddingScheme::PKCS1v15Encrypt, data)
                    .map_err(|e| SecureError::DecryptError(e.to_string()))
            }
            _ => Err(SecureError::Unsupported)
        }
    }

    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>, SecureError> {
        match self {
            Self::RsaPkcs1Pem { secure_key: true, key, hasher } => {
                let key = RsaPrivateKey::from_pkcs1_pem(key)
                    .map_err(|e| SecureError::ParseKeyError(e.to_string()))?;
                let data = hasher.hash(data);
                let padding = PaddingScheme::PKCS1v15Sign { hash: hasher.as_ras_hash() };

                key.sign(padding, &data)
                    .map_err(|e| SecureError::SignError(e.to_string()))
            }
            _ => Err(SecureError::Unsupported)
        }
    }

    pub fn verify(&self, data: &[u8], signed_data: &[u8]) -> Result<bool, SecureError> {
        match self {
            Self::RsaPkcs1Pem { secure_key: false, key, hasher } => {
                let key = RsaPublicKey::from_pkcs1_pem(key)
                    .map_err(|e| SecureError::ParseKeyError(e.to_string()))?;
                let data = hasher.hash(data);
                let padding = PaddingScheme::PKCS1v15Sign { hash: hasher.as_ras_hash() };

                key.verify(padding, &data, signed_data)
                    .map(|_| true)
                    .or_else(|e| {
                        match e {
                            rsa::errors::Error::Verification => Ok(false),
                            _ => Err(SecureError::VerifyError(e.to_string()))
                        }
                    })
            }
            _ => Err(SecureError::Unsupported)
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Hasher {
    None,
    SHA2_256,
    SHA2_512,
    SHA3_256,
    SHA3_512,
}

impl Hasher {
    pub fn hash(&self, data: &[u8]) -> Vec<u8> {
        match self {
            Hasher::None => data.to_vec(),
            Hasher::SHA2_256 => sha2::Sha256::digest(data).to_vec(),
            Hasher::SHA2_512 => sha2::Sha512::digest(data).to_vec(),
            Hasher::SHA3_256 => sha3::Sha3_256::digest(data).to_vec(),
            Hasher::SHA3_512 => sha3::Sha3_512::digest(data).to_vec(),
        }
    }

    pub fn as_ras_hash(&self) -> Option<Hash> {
        match self {
            Hasher::None => None,
            Hasher::SHA2_256 => Some(Hash::SHA2_256),
            Hasher::SHA2_512 => Some(Hash::SHA2_512),
            Hasher::SHA3_256 => Some(Hash::SHA3_256),
            Hasher::SHA3_512 => Some(Hash::SHA3_512),
        }
    }
}

impl Default for Hasher {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum SecureError {
    #[error("不支持该操作")]
    Unsupported,
    #[error("输入密钥/公钥格式错误: {0}")]
    ParseKeyError(String),
    #[error("加密失败: {0}")]
    EncryptError(String),
    #[error("解密失败: {0}")]
    DecryptError(String),
    #[error("签名失败: {0}")]
    SignError(String),
    #[error("验证失败: {0}")]
    VerifyError(String),
}
