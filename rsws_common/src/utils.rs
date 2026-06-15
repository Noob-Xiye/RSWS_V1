use base64::Engine;

/// Generate API Key (as signing key)
///
/// Format: ak_ + base64(random 32 bytes)
/// Frontend holds this value for computing signatures, NOT sent with requests.
pub fn generate_api_key() -> String {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};
    let mut rng = StdRng::from_os_rng();
    let api_key_bytes: [u8; 32] = rng.random();
    format!(
        "ak_{}",
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(api_key_bytes)
    )
}
