use bns_core::channels::wasm::CbChannel;
use bns_core::ecc::SecretKey;
use bns_core::types::channel::Channel;
use std::default::Default;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct SwarmConfig {
    pub stun: String,
    pub key: SecretKey,
}

impl Default for SwarmConfig {
    fn default() -> Self {
        Self {
            stun: "stun:stun.l.google.com:19302".to_string(),
            key: SecretKey::random(),
        }
    }
}
