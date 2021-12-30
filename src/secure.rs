use deku::bitvec::{BitSlice, BitVec, Msb0};
use deku::ctx::{Limit, Size};
use deku::DekuRead;
use rsa::{Hash, PaddingScheme, PublicKey, PublicKeyParts, RsaPrivateKey, RsaPublicKey};
use rsa::pkcs1::{FromRsaPrivateKey, FromRsaPublicKey};
use sha2::Digest;

use crate::error::WriteBinError;
use crate::ReadBinError;

const UNSUPPORTED: &'static str = "不支持该操作";

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "format")]
pub enum SecureKey {
    /// 不进行加密
    /// - 加解密时返回原始数据
    /// - 签名结果始终为空
    /// - 验证结果始终为通过
    None,
    /// RSA加密
    /// - 公/私钥格式为PKCS1 PEM
    /// - 加/解密时Padding为`PKCS1v15`
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
    pub fn rsa_pkcs1_pem<S: Into<String>>(key: S, is_secure_key: bool, hasher: Hasher) -> Self {
        Self::RsaPkcs1Pem {
            secure_key: is_secure_key,
            key: key.into(),
            hasher,
        }
    }
}

impl SecureKey {
    pub fn encrypt(&self, data: BitVec<Msb0, u8>) -> Result<BitVec<Msb0, u8>, WriteBinError> {
        match self {
            Self::None => Ok(data),
            Self::RsaPkcs1Pem { secure_key: false, key, .. } => {
                if data.len() % 8 != 0 {
                    return Err(WriteBinError::EncryptError(format!("加密数据必须全部为完整字节")));
                }
                let pk = RsaPublicKey::from_pkcs1_pem(key)
                    .map_err(|e| WriteBinError::EncryptError(e.to_string()))?;

                let data = data.as_raw_slice();
                let chunks_size = pk.size() - 11;
                let m = data.len() % chunks_size;
                let mut ret = Vec::with_capacity(data.len() + if m == 0 { 0 } else { chunks_size - m });

                for chunk in data.chunks(chunks_size) {
                    let mut en_data = pk
                        .encrypt(&mut rand::rngs::OsRng, PaddingScheme::PKCS1v15Encrypt, chunk)
                        .map_err(|e| WriteBinError::EncryptError(e.to_string()))?;
                    ret.append(&mut en_data);
                }
                Ok(BitVec::from_vec(ret))
            }
            _ => Err(WriteBinError::EncryptError(UNSUPPORTED.to_string()))
        }
    }


    pub fn sign(&self, data: &BitVec<Msb0, u8>) -> Result<BitVec<Msb0, u8>, WriteBinError> {
        match self {
            Self::None => Ok(BitVec::new()),
            Self::RsaPkcs1Pem { secure_key: true, key, hasher } => {
                if data.len() % 8 != 0 {
                    return Err(WriteBinError::SignError(format!("签名数据必须全部为完整字节")));
                }
                let key = RsaPrivateKey::from_pkcs1_pem(key)
                    .map_err(|e| WriteBinError::SignError(e.to_string()))?;
                let data = hasher.hash(data.as_raw_slice());
                let padding = PaddingScheme::PKCS1v15Sign { hash: hasher.as_ras_hash() };

                key.sign(padding, &data)
                    .map(|d| BitVec::from_vec(d))
                    .map_err(|e| WriteBinError::SignError(e.to_string()))
            }
            _ => Err(WriteBinError::SignError(UNSUPPORTED.to_string()))
        }
    }

    pub fn decrypt(&self, data: &BitSlice<Msb0, u8>) -> Result<BitVec<Msb0, u8>, ReadBinError> {
        match self {
            Self::None => Ok(data.to_bitvec()),
            Self::RsaPkcs1Pem { secure_key: true, key, .. } => {
                assert_eq!(data.len() % 8, 0, "解密数据必须全部为完整字节");
                let (_, data) = Vec::<u8>::read(data, Limit::new_size(Size::Bits(data.len())))?;

                let sk = RsaPrivateKey::from_pkcs1_pem(key)
                    .map_err(|e| ReadBinError::DecryptError(e.to_string()))?;
                let mut ret = Vec::with_capacity(data.len() / sk.size() * (sk.size() - 11));
                for chunk in data.chunks(sk.size()) {
                    let mut de_data = sk
                        .decrypt(PaddingScheme::PKCS1v15Encrypt, chunk)
                        .map_err(|e| ReadBinError::DecryptError(e.to_string()))?;
                    ret.append(&mut de_data);
                }
                Ok(BitVec::from_vec(ret))
            }
            _ => Err(ReadBinError::DecryptError(UNSUPPORTED.to_string()))
        }
    }

    pub fn verify(&self, data: &[u8], signed_data: &[u8]) -> Result<(), ReadBinError> {
        match self {
            Self::None => Ok(()),
            Self::RsaPkcs1Pem { secure_key: false, key, hasher } => {
                let key = RsaPublicKey::from_pkcs1_pem(key)
                    .map_err(|e| ReadBinError::VerifyError(e.to_string()))?;
                let data = hasher.hash(data);
                let padding = PaddingScheme::PKCS1v15Sign { hash: hasher.as_ras_hash() };

                key.verify(padding, &data, signed_data)
                    .map_err(|e| ReadBinError::VerifyError(e.to_string()))
            }
            _ => Err(ReadBinError::VerifyError(UNSUPPORTED.to_string()))
        }
    }
}

impl Default for SecureKey {
    fn default() -> Self {
        Self::None
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

