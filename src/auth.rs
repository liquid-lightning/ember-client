//! ember's challenge-sign-register identity: a secp256k1 keypair derived
//! deterministically from the caller's BIP39 seed, used only to prove seed
//! ownership during `Register`/`Recover` and to request a device
//! certificate for everything after. This is **not** CLN's own node key —
//! that's derived server-side, straight from the submitted mnemonic text
//! (ember is custodial). Mirrors the *shape* of Greenlight's
//! challenge/signature/CSR flow without pulling in `gl-client`'s signer,
//! which is a full embedded HSM stack (see the crate-level docs).

use hkdf::Hkdf;
use rcgen::{CertificateParams, DnType, KeyPair, PKCS_ECDSA_P256_SHA256};
use secp256k1::ecdsa::Signature;
use secp256k1::{Message, PublicKey, Secp256k1, SecretKey};
use sha2::Sha256;

use crate::error::Error;

/// Domain-separation label for the HKDF expand step, so this key can never
/// collide with some other seed-derived key ember or a future caller might
/// mint for an unrelated purpose from the same seed.
const HKDF_INFO: &[u8] = b"ember-client/auth-identity/v1";

/// A node's auth identity: a secp256k1 keypair deterministically derived
/// from its BIP39 seed. Deriving it again from the same seed always
/// reproduces the same `node_id` — that's what lets `Recover` find the
/// node again.
pub struct AuthKey {
    secret: SecretKey,
    public: PublicKey,
}

impl AuthKey {
    /// `seed` is the raw BIP39 seed (e.g. `Mnemonic::to_seed(passphrase)`),
    /// not the mnemonic text itself.
    pub fn from_seed(seed: &[u8]) -> Result<Self, Error> {
        let hk = Hkdf::<Sha256>::new(None, seed);
        let mut okm = [0u8; 32];
        hk.expand(HKDF_INFO, &mut okm)
            .expect("32-byte output is within HKDF-SHA256's 255*32-byte limit");

        let secret = SecretKey::from_slice(&okm).map_err(Error::Auth)?;
        let public = PublicKey::from_secret_key(&Secp256k1::new(), &secret);
        Ok(Self { secret, public })
    }

    /// Compressed secp256k1 pubkey (33 bytes) — ember's `node_id`.
    pub fn node_id(&self) -> [u8; 33] {
        self.public.serialize()
    }

    /// Signs a server-issued challenge (expected to be exactly 32 bytes —
    /// random data, never attacker-controlled, so it's signed directly as
    /// an ECDSA message digest with no extra hashing/prefix).
    pub fn sign_challenge(&self, challenge: &[u8]) -> Result<[u8; 64], Error> {
        let msg = Message::from_digest_slice(challenge).map_err(Error::Auth)?;
        let sig: Signature = Secp256k1::new().sign_ecdsa(&msg, &self.secret);
        Ok(sig.serialize_compact())
    }
}

/// A freshly generated device identity: its own ECDSA P-256 keypair (never
/// reused, never derived from the seed — ember's CA counter-signs the
/// public half as a device certificate, and the private half never leaves
/// this process) plus the PEM-encoded CSR to submit with `Register`/
/// `Recover`.
pub struct DeviceCsr {
    pub csr_pem: String,
    pub key_pair: KeyPair,
}

/// Generates a fresh device keypair and PEM-encoded CSR. The CSR's own
/// subject carries no identifying information — ember's CA assigns the
/// device certificate's real CN itself (see `domain::ca::CertificateAuthority::
/// sign_csr`'s docs on why the CSR's requested subject is never trusted).
pub fn generate_device_csr() -> Result<DeviceCsr, Error> {
    let key_pair = KeyPair::generate_for(&PKCS_ECDSA_P256_SHA256).map_err(Error::Cert)?;
    let mut params = CertificateParams::default();
    params
        .distinguished_name
        .push(DnType::CommonName, "ember-device");
    let csr = params.serialize_request(&key_pair).map_err(Error::Cert)?;
    let csr_pem = csr.pem().map_err(Error::Cert)?;

    Ok(DeviceCsr { csr_pem, key_pair })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_derives_the_same_node_id() {
        let seed = [7u8; 32];
        let a = AuthKey::from_seed(&seed).unwrap();
        let b = AuthKey::from_seed(&seed).unwrap();
        assert_eq!(a.node_id(), b.node_id());
    }

    #[test]
    fn different_seeds_derive_different_node_ids() {
        let a = AuthKey::from_seed(&[1u8; 32]).unwrap();
        let b = AuthKey::from_seed(&[2u8; 32]).unwrap();
        assert_ne!(a.node_id(), b.node_id());
    }

    #[test]
    fn signature_verifies_against_the_derived_node_id() {
        let key = AuthKey::from_seed(&[42u8; 32]).unwrap();
        let challenge = [9u8; 32];
        let sig = key.sign_challenge(&challenge).unwrap();

        let pubkey = PublicKey::from_slice(&key.node_id()).unwrap();
        let msg = Message::from_digest_slice(&challenge).unwrap();
        let signature = Signature::from_compact(&sig).unwrap();
        Secp256k1::verification_only()
            .verify_ecdsa(&msg, &signature, &pubkey)
            .expect("signature must verify against this key's own node_id");
    }
}
