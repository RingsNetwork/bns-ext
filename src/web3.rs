use wasm_bindgen::prelude::*;
use web3::api::Web3;
use web3::transports::eip_1193::Eip1193;
use web3::transports::eip_1193::Provider;

//eip 1193
#[wasm_bindgen(
    inline_js = "export function get_provider_js() { let provider = window.ethereum; if (!provider) {throw 'provider not found'}; return provider;}"
)]
extern "C" {
    #[wasm_bindgen(catch)]
    pub fn get_provider_js() -> Result<Option<Provider>, JsValue>;
}

pub struct Web3Provider(Web3<Eip1193>);

impl Web3Provider {
    pub fn new() -> Option<Self> {
        match get_provider_js() {
            Ok(Some(p)) => Some(Self(Web3::new(Eip1193::new(p)))),
            _ => None,
        }
    }
}
