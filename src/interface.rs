use crate::discovery::SwarmConfig;
use crate::web3::Web3Provider;

use anyhow::anyhow;
use anyhow::Result;
use bns_core::transports::Transport;
use bns_core::dht::Chord;
use bns_core::ecc::SecretKey;
use bns_core::message::handler::MessageHandler;
use bns_core::swarm::Swarm;
use bns_core::swarm::TransportManager;
use bns_core::types::ice_transport::IceTrickleScheme;
use futures::lock::Mutex;
use std::sync::Arc;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use web_sys::RtcSdpType;
use yew::prelude::*;
use yew::NodeRef;

pub struct MainView {
    pub swarm: Arc<Swarm>,
    pub web3: Option<Web3Provider>,
    pub key: SecretKey,
    pub msg_handler: Arc<MessageHandler>,
    pub pending_transport: Arc<Mutex<Option<Arc<Transport>>>>,
    pub current_sdp: Arc<std::sync::Mutex<Option<String>>>,
    sdp_input_ref: NodeRef,
    http_input_ref: NodeRef,
    sdp_textarea_ref: NodeRef
}

pub enum Msg {
    ConnectPeerViaHTTP(String),
    ConnectPeerViaICE(String),
    GenerateSdp,
    Update,
    None,
}

impl MainView {
    pub fn new(cfg: &SwarmConfig) -> Self {
        let dht = Arc::new(Mutex::new(Chord::new(cfg.key.address().into())));
        let swarm = Arc::new(Swarm::new(&cfg.stun, cfg.key));
        let msg_handler = Arc::new(MessageHandler::new(Arc::clone(&dht), swarm.clone()));
        Self {
            swarm: Arc::clone(&swarm),
            msg_handler: Arc::clone(&msg_handler),
            web3: Web3Provider::new(),
            key: cfg.key,
            pending_transport: Arc::new(Mutex::new(None)),
            current_sdp: Arc::new(std::sync::Mutex::new(None)),
            sdp_input_ref: NodeRef::default(),
            http_input_ref: NodeRef::default(),
            sdp_textarea_ref: NodeRef::default()
        }
    }

    pub fn listen(&self) {
        let msg_handler = Arc::clone(&self.msg_handler);

        let handler = Arc::clone(&msg_handler);
        let handler = Arc::clone(&handler);
        spawn_local(Box::pin(async move {
            handler.listen().await;
        }));
    }

    pub async fn trickle_handshake(
        swarm: Arc<Swarm>,
        key: SecretKey,
        url: String,
    ) -> Result<String> {
        let client = reqwest_wasm::Client::new();
        let transport = swarm.new_transport().await?;
        let req = transport.get_handshake_info(key, RtcSdpType::Offer).await?;
        match client
            .post(&url)
            .body(TryInto::<String>::try_into(req)?)
            .send()
            .await?
            .text()
            .await
        {
            Ok(resp) => {
                log::debug!("get answer and candidate from remote");
                let addr = transport
                    .register_remote_info(String::from_utf8(resp.as_bytes().to_vec())?.try_into()?)
                    .await?;
                swarm.register(&addr, Arc::clone(&transport)).await?;
                Ok("ok".to_string())
            }
            Err(e) => {
                log::error!("someting wrong {:?}", e);
                anyhow::Result::Err(anyhow!(e))
            }
        }
    }
}

impl Component for MainView {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let ret = Self::new(&SwarmConfig::default());
//        ret.listen();
        ret
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ConnectPeerViaHTTP(url) => {
                let swarm = Arc::clone(&self.swarm);
                let key = self.key;
                spawn_local(async move {
                    match Self::trickle_handshake(swarm, key, url).await {
                        Ok(s) => log::info!("{:?}", s),
                        Err(e) => {
                            log::error!("{:?}", e);
                        }
                    }
                });
                true
            }
            Msg::ConnectPeerViaICE(_sdp) => false,
            Msg::GenerateSdp => {
                let swarm = Arc::clone(&self.swarm);
                let pending = Arc::clone(&self.pending_transport);
                let sec_key = self.key.clone();
                let current_sdp = Arc::clone(&self.current_sdp);
                let link = ctx.link().clone();
                spawn_local(async move {
                    match swarm.new_transport().await {
                        Ok(t) => {
                            match t.get_handshake_info(sec_key, web_sys::RtcSdpType::Offer).await {
                                Ok(sdp) => {
                                    log::debug!("setting sdp area");
                                    let mut p = pending.lock().await;
                                    let mut s = current_sdp.lock().unwrap();
                                    *p = Some(Arc::clone(&t));
                                    *s = Some(sdp.to_string());
                                    drop(p);
                                    drop(s);
                                    log::debug!("done setting sdp area");
                                    link.send_message(Msg::Update);
                                },
                                Err(e) => {
                                    log::error!("cannot generate sdp offer {:?}", e)
                                }
                            }
                        },
                        Err(_) => {
                            log::error!("failed to setting pending transport")
                        }
                    }
                });
                log::debug!("should update");
                false
            }
            Msg::None => false,
            Msg::Update => {
                log::debug!("force update!");
                true
            }
        }
    }

    fn changed(&mut self, _ctx: &Context<Self>) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <body>
                <div id="viewport">
                <p>
                <input ref={self.sdp_input_ref.clone()} id="remote_sdp_field" type="text" />
                <button onclick={
                    let input = self.http_input_ref.clone();
                    ctx.link().callback(move |_| {
                        match input.cast::<HtmlInputElement>() {
                            Some(input) => Msg::ConnectPeerViaICE(input.value()),
                            None => Msg::None
                        }
                    })
                }>{"Connect with SDP Swap"}</button>
                </p>
                <p>
                <input ref={self.http_input_ref.clone()}id="remote_http_field" type="text" />
                <button onclick={
                    let input = self.http_input_ref.clone();
                    ctx.link().callback(move |_| {
                        match input.cast::<HtmlInputElement>() {
                            Some(input) => Msg::ConnectPeerViaHTTP(input.value()),
                            None => Msg::None
                        }
                    })
                }>{"Connect To Entry Node"}</button>
                </p>
                <p>
                <pre ref={self.sdp_textarea_ref.clone()}>{
                    (*self.current_sdp.lock().unwrap()).as_ref().unwrap_or(&"".to_string())
                }</pre>
                <button onclick={ctx.link().callback(move |_| Msg::GenerateSdp)}>{"Generate Handshake SDP"}</button>
                </p>
                </div>
            </body>
        }
    }
}
