use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::fmt::Display;
use core::fmt::Formatter;
use core::fmt::Result as FmtResult;
use core::ops::Deref;
use crypto::key_box::SecretKey;
use crypto::rand::OsRng;
use crypto::signers::hmac;

use crate::error::CryptoError;
use crate::error::Result;
use crate::jwa::HashAlgorithm;
use crate::jwk::Jwk;
use crate::jwk::JwkOperation;
use crate::jwk::JwkParams;
use crate::jwk::JwkParamsOct;
use crate::jwk::JwkType;
use crate::jwk::JwkUse;
use crate::jws::JwsAlgorithm;
use crate::jws::JwsSigner;
use crate::jws::JwsVerifier;
use crate::utils::decode_b64;
use crate::utils::encode_b64;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[allow(non_camel_case_types)]
pub enum HmacAlgorithm {
  /// HMAC using SHA-256
  HS256,
  /// HMAC using SHA-384
  HS384,
  /// HMAC using SHA-512
  HS512,
}

impl HmacAlgorithm {
  pub const fn name(self) -> &'static str {
    match self {
      Self::HS256 => "HS256",
      Self::HS384 => "HS384",
      Self::HS512 => "HS512",
    }
  }

  pub const fn hash_alg(self) -> HashAlgorithm {
    match self {
      Self::HS256 => HashAlgorithm::Sha256,
      Self::HS384 => HashAlgorithm::Sha384,
      Self::HS512 => HashAlgorithm::Sha512,
    }
  }

  /// Creates a new Hmac key.
  pub fn generate_key(self) -> Result<SecretKey> {
    SecretKey::random(self.hash_alg().output_size(), &mut OsRng).map_err(Into::into)
  }

  /// Creates a new `HmacSigner` from a slice of bytes.
  pub fn signer_from_bytes(self, data: impl AsRef<[u8]>) -> Result<HmacSigner> {
    let data: &[u8] = data.as_ref();

    if data.len() < self.hash_alg().output_size() {
      return Err(CryptoError::InvalidKeyFormat(self.name()).into());
    }

    Ok(HmacSigner {
      alg: self,
      key: data.into(),
      kid: None,
    })
  }

  /// Creates a new `HmacSigner` from a base64url-encoded key.
  pub fn signer_from_b64(self, data: impl AsRef<[u8]>) -> Result<HmacSigner> {
    let data: Vec<u8> = decode_b64(data.as_ref())?;

    if data.len() < self.hash_alg().output_size() {
      return Err(CryptoError::InvalidKeyFormat(self.name()).into());
    }

    Ok(HmacSigner {
      alg: self,
      key: data.into(),
      kid: None,
    })
  }

  /// Creates a new `HmacSigner` from a JSON Web Key.
  pub fn signer_from_jwk(self, data: &Jwk) -> Result<HmacSigner> {
    data.check_use(&JwkUse::Signature)?;
    data.check_ops(&JwkOperation::Sign)?;
    data.check_alg(self.name())?;
    data.check_kty(JwkType::Oct)?;

    let k: Vec<u8> = match data.params() {
      Some(JwkParams::Oct(JwkParamsOct { k })) => decode_b64(k)?,
      Some(_) => return Err(CryptoError::InvalidKeyFormat(self.name()).into()),
      None => return Err(CryptoError::InvalidKeyFormat(self.name()).into()),
    };

    self.signer_from_bytes(k).map(|mut signer| {
      signer.kid = data.kid().map(ToString::to_string);
      signer
    })
  }

  /// Creates a new `HmacVerifier` from a slice of bytes.
  pub fn verifier_from_bytes(self, data: impl AsRef<[u8]>) -> Result<HmacVerifier> {
    let data: &[u8] = data.as_ref();

    if data.len() < self.hash_alg().output_size() {
      return Err(CryptoError::InvalidKeyFormat(self.name()).into());
    }

    Ok(HmacVerifier {
      alg: self,
      key: data.into(),
      kid: None,
    })
  }

  /// Creates a new `HmacVerifier` from a base64url-encoded key.
  pub fn verifier_from_b64(self, data: impl AsRef<[u8]>) -> Result<HmacVerifier> {
    let data: Vec<u8> = decode_b64(data.as_ref())?;

    if data.len() < self.hash_alg().output_size() {
      return Err(CryptoError::InvalidKeyFormat(self.name()).into());
    }

    Ok(HmacVerifier {
      alg: self,
      key: data.into(),
      kid: None,
    })
  }

  /// Creates a new `HmacVerifier` from a JSON Web Key.
  pub fn verifier_from_jwk(self, data: &Jwk) -> Result<HmacVerifier> {
    data.check_use(&JwkUse::Signature)?;
    data.check_ops(&JwkOperation::Verify)?;
    data.check_alg(self.name())?;
    data.check_kty(JwkType::Oct)?;

    let k: Vec<u8> = match data.params() {
      Some(JwkParams::Oct(JwkParamsOct { k })) => decode_b64(k)?,
      Some(_) => return Err(CryptoError::InvalidKeyFormat(self.name()).into()),
      None => return Err(CryptoError::InvalidKeyFormat(self.name()).into()),
    };

    self.verifier_from_bytes(k).map(|mut signer| {
      signer.kid = data.kid().map(ToString::to_string);
      signer
    })
  }

  /// Creates a new Hmac JSON Web Key with the given slice of bytes.
  pub fn to_jwk(self, data: impl AsRef<[u8]>) -> Jwk {
    let mut jwk: Jwk = Jwk::with_kty(JwkType::Oct);
    let mut ops: Vec<JwkOperation> = Vec::with_capacity(2);

    ops.push(JwkOperation::Sign);
    ops.push(JwkOperation::Verify);

    jwk.set_alg(self.name());
    jwk.set_use(JwkUse::Signature);
    jwk.set_key_ops(ops);

    jwk.set_params(JwkParamsOct {
      k: encode_b64(data),
    });

    jwk
  }
}

impl Display for HmacAlgorithm {
  fn fmt(&self, f: &mut Formatter) -> FmtResult {
    f.write_fmt(format_args!("{}", self.name()))
  }
}

impl From<HmacAlgorithm> for JwsAlgorithm {
  fn from(other: HmacAlgorithm) -> Self {
    match other {
      HmacAlgorithm::HS256 => Self::HS256,
      HmacAlgorithm::HS384 => Self::HS384,
      HmacAlgorithm::HS512 => Self::HS512,
    }
  }
}

// =============================================================================
// Hmac Signer
// =============================================================================

#[derive(Debug)]
pub struct HmacSigner {
  alg: HmacAlgorithm,
  key: SecretKey,
  kid: Option<String>,
}

impl JwsSigner for HmacSigner {
  fn alg(&self) -> JwsAlgorithm {
    self.alg.into()
  }

  fn kid(&self) -> Option<&str> {
    self.kid.as_deref()
  }

  fn sign(&self, message: &[u8]) -> Result<Vec<u8>> {
    let signature: Vec<u8> = match self.alg {
      HmacAlgorithm::HS256 => hmac::sign_sha256(&self.key, message)?.to_vec(),
      HmacAlgorithm::HS384 => hmac::sign_sha384(&self.key, message)?.to_vec(),
      HmacAlgorithm::HS512 => hmac::sign_sha512(&self.key, message)?.to_vec(),
    };

    Ok(signature)
  }
}

impl Deref for HmacSigner {
  type Target = dyn JwsSigner;

  fn deref(&self) -> &Self::Target {
    self
  }
}

// =============================================================================
// Hmac Verifier
// =============================================================================

#[derive(Debug)]
pub struct HmacVerifier {
  alg: HmacAlgorithm,
  key: SecretKey,
  kid: Option<String>,
}

impl JwsVerifier for HmacVerifier {
  fn alg(&self) -> JwsAlgorithm {
    self.alg.into()
  }

  fn kid(&self) -> Option<&str> {
    self.kid.as_deref()
  }

  fn verify(&self, message: &[u8], signature: &[u8]) -> Result<()> {
    match self.alg {
      HmacAlgorithm::HS256 => hmac::verify_sha256(&self.key, message, signature)?,
      HmacAlgorithm::HS384 => hmac::verify_sha384(&self.key, message, signature)?,
      HmacAlgorithm::HS512 => hmac::verify_sha512(&self.key, message, signature)?,
    }

    Ok(())
  }
}

impl Deref for HmacVerifier {
  type Target = dyn JwsVerifier;

  fn deref(&self) -> &Self::Target {
    self
  }
}
