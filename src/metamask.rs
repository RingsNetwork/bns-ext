// EIP 1193 implementation for metamask ext
// ref: https://github.com/tomusdrw/rust-web3/blob/master/src/transports/eip_1193.rs
// ref: https://eips.ethereum.org/EIPS/eip-1193

use anyhow::anyhow;
use anyhow::Result;
use wasm_bindgen::prelude::*;

use futures::channel::mpsc;

use std::{cell::RefCell, collections::BTreeMap, rc::Rc};
use web3::api::SubscriptionId;

type Subscriptions =
    Rc<RefCell<BTreeMap<SubscriptionId, mpsc::UnboundedSender<serde_json::Value>>>>;

#[derive(Clone, Debug)]
pub struct Provider(Port);

pub enum Browser {
    Chrome,
    Firefox,
}

#[wasm_bindgen]
extern "C" {
    #[derive(Clone, Debug)]
    /// An EIP-1193 provider object. Available by convention at `window.ethereum`
    pub type Runtime;

    #[derive(Clone, Debug)]
    /// An EIP-1193 provider object. Available by convention at `window.ethereum`
    pub type Port;

    #[wasm_bindgen(
        catch,
        inline_js = "export function browser() { try { browser.runtime }}"
    )]
    pub fn runtime() -> Result<Runtime, JsValue>;

    #[wasm_bindgen(method)]
    fn connect(this: &Runtime, ext_id: String) -> Port;
}

// https://github.com/MetaMask/extension-provider/blob/master/config.json
pub fn get_metamask_id(browser: Browser) -> String {
    match browser {
        Browser::Chrome => "nkbihfbeogaeaoehlefnkodbefgpgknn".to_string(),
        Browser::Firefox => "webextension@metamask.io".to_string(),
    }
}

impl Provider {
    pub fn new(b: Browser) -> Result<Self> {
        match runtime() {
            Ok(r) => Ok(Self(r.connect(get_metamask_id(b)))),
            Err(e) => {
                log::error!("failed on get ext runtime");
                Err(anyhow!("{:?}", e))
            }
        }
    }
}

/// EIP-1193 transport
#[derive(Clone, Debug)]
pub struct Eip1193 {
    provider_and_listeners: Rc<RefCell<ProviderAndListeners>>,
    subscriptions: Subscriptions,
}

/// Keep the provider and the event listeners attached to it together so we can remove them in the
/// `Drop` implementation. The logic can't go in Eip1193 because it's `Clone`, and cloning a JS
/// object just clones the reference.
#[derive(Debug)]
pub struct ProviderAndListeners {
    pub provider: Provider,
    pub listeners: BTreeMap<String, Vec<Closure<dyn FnMut(JsValue)>>>,
}
