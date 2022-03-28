use bns_core::ecc::SecretKey;

use std::default::Default;

#[derive(Clone, Debug)]
pub struct SwarmConfig {
    pub stun: String,
    pub key: SecretKey,
}

impl Default for SwarmConfig {
    fn default() -> Self {
        Self {
            stun: "turn://bns:password@127.0.0.1:3478".to_string(),
            key: SecretKey::random(),
        }
    }
}
