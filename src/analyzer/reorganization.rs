//! This module aims to the instant chain reorganization events.
//! This module produces the below measurements:
//!   - [Reorganization](TODO link to Reorganization)
//!
//! ### How it works?
//!
//! There is an thread subscribed to CKB on the topic "NewTipHeader"(see more about CKB subscription
//! interface from [..](TODO link to ckb subscribe). This subscription will notify us whenever the
//! tip changes. Via observing the tip changing, we construct the last part of main chain. Then we
//! can observe a chain reorganization occurs by comparing the arrival tip change with our
//! constructed main chain.
//!
//! ### Why measure it?
//!
//! Monitoring to chain reorganization may help to:
//!   - detect selfish mining attacks
//!   - understand the network between miners

// TODO rename main_ to canonical_; remove prefix main_ from main_tip_hash/main_tip_number

use crate::measurement::{self, IntoWriteQuery};
use crate::subscribe::{Subscription, Topic};
use ckb_suite_rpc::Jsonrpc;
use ckb_types::core::{BlockNumber, HeaderView};
use ckb_types::packed::Byte32;
use crossbeam::channel::Sender;
use influxdb::{Timestamp, WriteQuery};
use jsonrpc_core::futures::Stream;
use jsonrpc_core::serde_from_str;
use jsonrpc_server_utils::tokio::prelude::*;

pub struct Reorganization {
    header_receiver: jsonrpc_server_utils::tokio::sync::mpsc::Receiver<String>,
    query_sender: Sender<WriteQuery>,
    jsonrpc: Jsonrpc,
    main_tip_number: BlockNumber,
    main_tip_hash: Byte32,
}

impl Reorganization {
    pub fn new(
        ckb_rpc_url: String,
        ckb_subscribe_url: String,
        query_sender: Sender<WriteQuery>,
    ) -> (Self, Subscription) {
        let jsonrpc = Jsonrpc::connect(&ckb_rpc_url);
        let (header_sender, header_receiver) =
            jsonrpc_server_utils::tokio::sync::mpsc::channel(100);
        let subscription = Subscription::new(ckb_subscribe_url, Topic::NewTipHeader, header_sender);
        (
            Self {
                jsonrpc,
                header_receiver,
                query_sender,
                main_tip_number: 0,
                main_tip_hash: Default::default(),
            },
            subscription,
        )
    }

    pub async fn run(mut self) {
        log::info!("{} started ...", ::std::any::type_name::<Self>());

        // Take out the header_receiver to pass the Rust borrow rule
        let (_, mut dummy_receiver) = jsonrpc_server_utils::tokio::sync::mpsc::channel(100);
        ::std::mem::swap(&mut self.header_receiver, &mut dummy_receiver);
        let header_receiver = dummy_receiver;

        header_receiver
            .for_each(|message| {
                let header: ckb_suite_rpc::ckb_jsonrpc_types::HeaderView = serde_from_str(&message)
                    .unwrap_or_else(|err| {
                        panic!("serde_from_str(\"{}\"), error: {:?}", message, err)
                    });
                let header: HeaderView = header.into();
                self.handle(&header);
                Ok(())
            })
            .wait()
            .unwrap_or_else(|err| panic!("receiver error {:?}", err));
    }

    fn handle(&mut self, header: &HeaderView) {
        if self.main_tip_hash == header.parent_hash() || self.main_tip_number == 0 {
            self.main_tip_hash = header.hash();
            self.main_tip_number = header.number();
        } else {
            let new_tip = header;
            let old_tip = self.get_header(self.main_tip_hash.clone());
            let ancestor = self.locate_ancestor(&old_tip, new_tip);
            self.report_reorganization(new_tip, &old_tip, &ancestor);

            self.main_tip_hash = header.hash();
            self.main_tip_number = header.number();
        }
    }

    fn locate_ancestor(&mut self, old_tip: &HeaderView, new_tip: &HeaderView) -> HeaderView {
        let mut old_tip = old_tip.clone();
        let mut new_tip = new_tip.clone();
        #[allow(clippy::comparison_chain)]
        if old_tip.number() > new_tip.number() {
            for _ in 0..old_tip.number() - new_tip.number() {
                old_tip = self.get_header(old_tip.parent_hash());
            }
        } else if old_tip.number() < new_tip.number() {
            for _ in 0..new_tip.number() - old_tip.number() {
                new_tip = self.get_header(new_tip.parent_hash());
            }
        }
        assert_eq!(old_tip.number(), new_tip.number());
        while old_tip.hash() != new_tip.hash() {
            old_tip = self.get_header(old_tip.parent_hash());
            new_tip = self.get_header(new_tip.parent_hash());
        }
        old_tip
    }

    fn report_reorganization(
        &mut self,
        new_tip: &HeaderView,
        old_tip: &HeaderView,
        ancestor: &HeaderView,
    ) {
        let attached_length = new_tip.number() - ancestor.number();
        log::info!(
            "Reorganize from #{}({:#x}) to #{}({:#x}), attached_length = {}",
            old_tip.number(),
            old_tip.hash(),
            new_tip.number(),
            new_tip.hash(),
            attached_length
        );
        let query = measurement::Reorganization {
            time: Timestamp::Milliseconds(ancestor.timestamp() as u128),
            attached_length: attached_length as u32,
            old_tip_number: old_tip.number(),
            old_tip_hash: format!("{:#x}", old_tip.hash()),
            new_tip_number: new_tip.number(),
            new_tip_hash: format!("{:#x}", new_tip.hash()),
            ancestor_number: ancestor.number(),
            ancestor_hash: format!("{:#x}", ancestor.hash()),
        }
        .into_write_query();
        self.query_sender.send(query).unwrap();
    }

    fn get_header(&mut self, block_hash: Byte32) -> HeaderView {
        if let Some(header) = self.jsonrpc.get_header(block_hash.clone()) {
            return header.into();
        }

        if let Some(block) = self.jsonrpc.get_fork_block(block_hash) {
            return block.header.into();
        }

        unreachable!()
    }
}
