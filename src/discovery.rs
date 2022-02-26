use anyhow::anyhow;
use bns_core::swarm::Swarm;
use bns_core::types::channel::Channel;
use bns_core::channels::wasm::CbChannel;
use bns_core::ecc::SecretKey;
use std::default::Default;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct SwarmConfig {
    pub stun: String,
    pub channel: Arc<CbChannel>,
    pub key: SecretKey
}

impl Default for SwarmConfig {
    fn default() -> Self {
        Self {
            stun: "stun:stun.l.google.com:19302".to_string(),
            channel: Arc::new(CbChannel::new(1)),
            key: SecretKey::random()
        }
    }
}

pub struct Peer {
    pub swarm: Swarm
}

impl Peer {
    pub fn new(cfg: &SwarmConfig) -> Self {
        Self {
            swarm: Swarm::new(
                Arc::clone(&cfg.channel),
                cfg.stun.to_owned(),
                cfg.key.address())
        }
    }
}
