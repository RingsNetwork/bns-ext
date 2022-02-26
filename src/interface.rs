use yew::prelude::*;
use std::sync::Arc;
use bns_core::swarm::Swarm;
use crate::discovery::SwarmConfig;
use crate::web3::Web3Provider;

pub struct MainView {
    pub swarm: Swarm,
    pub web3: Option<Web3Provider>
}

pub enum Msg {

}

impl MainView {
    pub fn new(cfg: &SwarmConfig) -> Self {
        Self {
            swarm: Swarm::new(
                Arc::clone(&cfg.channel),
                cfg.stun.to_owned(),
                cfg.key.address()),
            web3: Web3Provider::new()
        }
    }
}

impl Component for MainView {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self::new(&SwarmConfig::default())
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        true
    }

    fn changed(&mut self, _ctx: &Context<Self>) -> bool {
        false
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html!{
            <body>
                <div>
            {"hello bns"}
            </div>
            </body>
        }
    }
}
