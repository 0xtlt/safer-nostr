use secp256k1::{schnorr::Signature, XOnlyPublicKey, SECP256K1};
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SigError {
    #[error("Signature error: {0}")]
    SignatureError(#[from] secp256k1::Error),
}

pub fn verify_sig(sig: &str, pubkey: &str, message: &str) -> Result<(), SigError> {
    let sig = Signature::from_str(sig)?;
    let pubkey = XOnlyPublicKey::from_str(pubkey)?;
    let message: secp256k1::Message =
        secp256k1::Message::from_hashed_data::<secp256k1::hashes::sha256::Hash>(message.as_bytes());

    SECP256K1.verify_schnorr(&sig, &message, &pubkey)?;

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
    cache: &super::cache::Cache,
    pubkey: Option<&String>,
    sig: Option<&String>,
    time: Option<&String>,
    uniq: Option<&String>,
    pass: Option<&String>,
) -> bool {
    if crate::ENV_CONFIG.password.is_some()
        && (pass.is_none() || pass.unwrap().clone() != crate::ENV_CONFIG.password.clone().unwrap())
    {
        println!("Bad password");
        return false;
    }

    if crate::ENV_CONFIG.restricted_pubkeys.is_empty() {
        return true;
    }

    if pubkey.is_none() || sig.is_none() || time.is_none() || uniq.is_none() {
        println!("Invalid request");
        return false;
    }

    let pubkey = pubkey.unwrap();
    let sig = sig.unwrap();
    let time = time.unwrap();
    let uniq = uniq.unwrap();

    // Check if the pubkey is in the RESTRICTED_PUBKEYS list
    if !crate::ENV_CONFIG.restricted_pubkeys.contains(pubkey) {
        println!("Invalid pubkey: {pubkey}");
        return false;
    }

    // Check if the time is within 5 minutes of the current time
    let time_of_request = time.parse::<i64>().unwrap();
    let current_time = chrono::Utc::now().timestamp();

    if (time_of_request - current_time).abs() > 300 {
        println!("Invalid time: {time}");
        return false;
    }

    if verify_sig(sig, pubkey, &format!("{pubkey}:{time}:{uniq}")).is_err() {
        println!("Invalid signature");
        return false;
    }

    // Check if the signature is not used before
    let key = format!("sig:{pubkey}:{time}:{uniq}");
    if cache.to_owned().get_str(&key).await.is_ok() {
        return false;
    }

    // Set the signature to be used
    cache
        .to_owned()
        .to_owned()
        .set_str(&key, "1", crate::ENV_CONFIG.cache_ttl_signature)
        .await
        .unwrap();

    true
}
