use crate::subscribe::{Subscription, Topic};
use ckb_types::core::BlockNumber;
use ckb_types::packed::Byte32;
use jsonrpc_core::futures::Stream;
use jsonrpc_core::serde_from_str;
use jsonrpc_server_utils::tokio::{
    prelude::*,
    sync::mpsc::{channel, Receiver},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ForkConfig {
    pub ckb_subscribe_url: String,
}

pub struct Fork {
    // #{block_number => #{block_hash => parent_hash}}
    cache: HashMap<BlockNumber, HashMap<Byte32, Byte32>>,
    receiver: Receiver<String>,
}

impl Fork {
    pub fn init(config: ForkConfig) -> (Self, Subscription) {
        let (sender, receiver) = channel(100);
        let subscription = Subscription::new(config.ckb_subscribe_url, Topic::NewTipBlock, sender);
        (
            Self {
                receiver,
                cache: Default::default(),
            },
            subscription,
        )
    }

    pub async fn run(self) {
        println!("run {}", ::std::any::type_name::<Self>());
        self.receiver
            .for_each(|message| {
                let block: ckb_suite_rpc::ckb_jsonrpc_types::BlockView = serde_from_str(&message)
                    .unwrap_or_else(|err| {
                        panic!("serde_from_str(\"{}\"), error: {:?}", message, err)
                    });
                let block: ckb_types::core::BlockView = block.into();
                println!("fork run block: {}", block.number());
                Ok(())
            })
            .wait()
            .unwrap_or_else(|err| panic!("receiver error {:?}", err));
    }
}
