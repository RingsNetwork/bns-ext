// doc https://bugzilla.mozilla.org/show_bug.cgi?id=1659672

use crate::discovery::SwarmConfig;
use crate::web3::Web3Provider;
use anyhow::anyhow;
use anyhow::Result;
use bns_core::dht::Chord;
use bns_core::ecc::SecretKey;
use bns_core::message::handler::MessageHandler;
use bns_core::message::{Decoder, Encoder};
use bns_core::swarm::Swarm;
use bns_core::swarm::TransportManager;
use bns_core::transports::wasm::WasmTransport as Transport;
use bns_core::types::ice_transport::IceTrickleScheme;
use bns_core::types::message::MessageListener;
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
    pub current_offer: Arc<std::sync::Mutex<Option<String>>>,
    pub current_answer: Arc<std::sync::Mutex<Option<String>>>,
    offer_textarea: NodeRef,
    answer_textarea: NodeRef,
    http_input_ref: NodeRef,
}

pub enum Msg {
    ConnectPeerViaHTTP(String),
    ConnectPeerViaICE(String),
    ResponseOffer(String),
    AcceptAnswer(String),
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
            current_offer: Arc::new(std::sync::Mutex::new(None)),
            current_answer: Arc::new(std::sync::Mutex::new(None)),
            offer_textarea: NodeRef::default(),
            answer_textarea: NodeRef::default(),
            http_input_ref: NodeRef::default(),
        }
    }

    pub fn listen(&self) {
        let msg_handler = Arc::clone(&self.msg_handler);
        spawn_local(Box::pin(async move {
            let handler = Arc::clone(&msg_handler);
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
            .body(String::from_encoded(&req)?)
            .send()
            .await?
            .text()
            .await
        {
            Ok(resp) => {
                log::debug!("get answer and candidate from remote");
                let addr = transport
                    .register_remote_info(String::from_utf8(resp.as_bytes().to_vec())?.encode()?)
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
                let current_offer = Arc::clone(&self.current_offer);
                let link = ctx.link().clone();
                spawn_local(async move {
                    match swarm.new_transport().await {
                        Ok(t) => {
                            match t
                                .get_handshake_info(sec_key, web_sys::RtcSdpType::Offer)
                                .await
                            {
                                Ok(sdp) => {
                                    log::debug!("setting sdp offer area");
                                    let mut p = pending.lock().await;
                                    let mut s = current_offer.lock().unwrap();
                                    *p = Some(Arc::clone(&t));
                                    *s = Some(sdp.to_string());
                                    drop(p);
                                    drop(s);
                                    log::debug!("done setting sdp offer area");
                                    link.send_message(Msg::Update);
                                }
                                Err(e) => {
                                    log::error!("cannot generate sdp offer {:?}", e);
                                }
                            }
                        }
                        Err(_) => {
                            log::error!("failed to setting pending transport");
                        }
                    }
                });
                log::debug!("should update");
                false
            }
            Msg::ResponseOffer(s) => {
                log::debug!("get offer {:?}", s.clone());
                let swarm = Arc::clone(&self.swarm);
                let offer = s.clone();
                let pending = Arc::clone(&self.pending_transport);
                let current_answer = Arc::clone(&self.current_answer);
                let link = ctx.link().clone();
                let sec_key = self.key.clone();

                spawn_local(async move {
                    match swarm.new_transport().await {
                        Ok(t) => match t.register_remote_info(offer.encode().unwrap()).await {
                            Ok(addr) => {
                                let sdp = t
                                    .get_handshake_info(sec_key, web_sys::RtcSdpType::Answer)
                                    .await
                                    .unwrap();
                                swarm.register(&addr, t.clone()).await.unwrap();
                                log::debug!("setting sdp answer area");
                                let mut p = pending.lock().await;
                                let mut s = current_answer.lock().unwrap();
                                *p = Some(Arc::clone(&t));
                                *s = Some(sdp.to_string());
                                drop(p);
                                drop(s);
                                log::debug!("done setting sdp answer area");
                                link.send_message(Msg::Update);
                            }
                            Err(e) => {
                                log::error!("cannot generate sdp answer {:?}", e);
                            }
                        },
                        Err(e) => {
                            log::error!("cannot generate new transport {:?}", e);
                        }
                    };
                });
                false
            }
            Msg::AcceptAnswer(s) => {
                let pending = Arc::clone(&self.pending_transport);
                // if pending.is_none() {
                //     log::error!("cannot find pending transport, maybe you should create offer first");
                // }
                let swarm = Arc::clone(&self.swarm);
                let answer = s.clone();
                let link = ctx.link().clone();

                spawn_local(async move {
                    if let Some(t) = &*pending.lock().await {
                        let addr = t
                            .register_remote_info(answer.encode().unwrap())
                            .await
                            .unwrap();
                        swarm.register(&addr, t.clone()).await.unwrap();
                        link.send_message(Msg::Update);
                    }
                });
                false
            }
            Msg::Update => {
                log::debug!("force update!");
                true
            }
            _ => false,
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
                <input ref={self.http_input_ref.clone()} type="text" />
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
                <div>
                <h2>{"Handshake with SDP Exchange"}</h2>
                <p>
                <h3>{"Create Offer"}</h3>
                <button onclick={ctx.link().callback(move |_| Msg::GenerateSdp)}>{"Generate Handshake Offer"}</button>
                <pre>{
                    (*self.current_offer.lock().unwrap()).as_ref().unwrap_or(&"".to_string())
                }</pre>

                </p>
                <p>
                <h3>{"Responser and Create Answer"}</h3>
                <textarea ref={self.offer_textarea.clone()} type="text" ></textarea>
                <button onclick={
                    let input = self.offer_textarea.clone();
                    log::info!("select textarea {:?}", &input);
                    ctx.link().callback(move |_| {
                        match input.cast::<web_sys::HtmlTextAreaElement>() {
                            Some(textarea) => {
                                Msg::ResponseOffer(textarea.value())
                            },
                            None => Msg::None
                        }
                    })
                }>{"Response Offer"}</button>
                <pre>{
                    (*self.current_answer.lock().unwrap()).as_ref().unwrap_or(&"".to_string())
                }</pre>
                </p>

                <p>
                <h3>{"Accept Answer"}</h3>
                <textarea ref={self.answer_textarea.clone()} type="text" ></textarea>
                <button onclick={
                    let input = self.answer_textarea.clone();
                    ctx.link().callback(move |_| {
                        match input.cast::<web_sys::HtmlTextAreaElement>() {
                            Some(input) => Msg::AcceptAnswer(input.value()),
                            None => Msg::None
                        }
                    })
                }>{"Accept Answer"}</button>
                </p>

                </div>
                </div>
                <p>
                <textarea></textarea>
                <button>{"send message"}</button>
                </p>
            </body>
        }
    }
}
