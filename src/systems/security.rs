use super::cache::{Cache, DEFAULT_CACHE_TTL, SECURITY_SIG_CACHE_TTL};
use crate::{CACHE, RESTRICTED_PUBKEYS};
use secp256k1::{schnorr::Signature, XOnlyPublicKey, SECP256K1};
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SigError {
    #[error("Signature error: {0}")]
    SignatureError(#[from] secp256k1::Error),
}

pub fn verify_sig(content: &str, pubkey: &str, sig: &str) -> Result<(), SigError> {
    let message =
        secp256k1::Message::from_hashed_data::<secp256k1::hashes::sha256::Hash>(content.as_bytes());

    SECP256K1.verify_schnorr(
        &Signature::from_str(sig)?,
        &message,
        &XOnlyPublicKey::from_str(pubkey)?,
    )?;
    Ok(())
}

/// Check if the request is from a valid user
/// If RESTRICTED_PUBKEYS is empty, then all requests are valid
/// If RESTRICTED_PUBKEYS is not empty, then only requests from a valid pubkey are valid
/// To check if a request is valid, we check if the signature is valid
/// The signature will be generated by the client using the private key with the following format:
/// <pubkey>:<time>:<uniq>
///
/// - pubkey: the public key of the user
/// - time: the current timestamp in seconds
/// - uniq: a random string
///
/// Then it must be sent to the server with the following parameters:
/// - pubkey: the public key of the user
/// - sig: the signature generated by the client
/// - time: the above time
/// - uniq: the above uniq
///
/// The server will then check 4 conditions:
/// - The pubkey must be in the RESTRICTED_PUBKEYS list
/// - The time must be within 5 minutes of the current time
/// - The signature must be valid
/// - The signature must be not used before
pub async fn check_access(
    pubkey: Option<&String>,
    sig: Option<&String>,
    time: Option<&String>,
    uniq: Option<&String>,
) -> bool {
    if RESTRICTED_PUBKEYS.is_empty() {
        return true;
    }

    if pubkey.is_none() || sig.is_none() || time.is_none() || uniq.is_none() {
        return false;
    }

    let pubkey = pubkey.unwrap();
    let sig = sig.unwrap();
    let time = time.unwrap();
    let uniq = uniq.unwrap();

    // Check if the pubkey is in the RESTRICTED_PUBKEYS list
    if !RESTRICTED_PUBKEYS.contains(pubkey) {
        return false;
    }

    // Check if the time is within 5 minutes of the current time
    let time_of_request = time.parse::<i64>().unwrap();
    let current_time = chrono::Utc::now().timestamp();

    if (time_of_request - current_time).abs() > SECURITY_SIG_CACHE_TTL as i64 {
        return false;
    }

    // Check if the signature is valid
    let message = format!("{pubkey}:{time}:{uniq}");
    if verify_sig(&message, pubkey, sig).is_err() {
        return false;
    }

    // Check if the signature is not used before
    let key = format!("sig:{pubkey}:{time}:{uniq}");
    if CACHE.to_owned().get_str(&key).await.is_ok() {
        return false;
    }

    // Set the signature to be used
    CACHE
        .to_owned()
        .set_str(&key, "1", SECURITY_SIG_CACHE_TTL)
        .await
        .unwrap();

    true
}
