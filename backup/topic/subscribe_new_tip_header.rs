use crate::config::Config;
use crate::subscribe::{Subscription, Topic};
use crate::util::retry_send;
use ckb_suite_rpc::ckb_jsonrpc_types::HeaderView as JsonHeader;
use ckb_types::core::HeaderView;
use jsonrpc_core::serde_from_str;
use std::time::Duration;

pub(crate) struct SubscribeNewTipHeader {
    config: Config,
    subscriber: crossbeam::channel::Receiver<(Topic, String)>,
    query_sender: crossbeam::channel::Sender<String>,
    hostname: String,
}

impl SubscribeNewTipHeader {
    pub(crate) fn new(
        config: Config,
        query_sender: crossbeam::channel::Sender<String>,
    ) -> (Self, Subscription) {
        let hostname = gethostname::gethostname().to_string_lossy().to_string();
        let (subscription, subscriber) =
            Subscription::new(config.subscription_url(), Topic::NewTipHeader);
        (
            Self {
                config,
                query_sender,
                subscriber,
                hostname,
            },
            subscription,
        )
    }

    async fn try_recv_subscription(
        &self,
    ) -> Result<(Topic, String), crossbeam::channel::TryRecvError> {
        self.subscriber.try_recv()
    }

    pub(crate) async fn run(mut self) {
        loop {
            match self.try_recv_subscription().await {
                Ok((topic, message)) => {
                    assert_eq!(Topic::NewTipHeader, topic);
                    let header: JsonHeader = serde_from_str(&message).unwrap();
                    let header: HeaderView = header.into();
                    self.report_new_tip_header(&header).await;
                }
                Err(crossbeam::channel::TryRecvError::Disconnected) => return,
                Err(crossbeam::channel::TryRecvError::Empty) => {
                    tokio::time::sleep(Duration::from_secs(1)).await
                }
            }
        }
    }
    async fn report_new_tip_header(&mut self, header: &HeaderView) {
        let block_timestamp = chrono::NaiveDateTime::from_timestamp(
            (header.timestamp() / 1000) as i64,
            (header.timestamp() % 1000 * 1000) as u32,
        );
        let time = chrono::Utc::now().naive_utc();
        let point = crate::table::SubscribeNewTipHeader {
            network: self.config.network(),
            time,
            hostname: self.hostname.clone(),
            block_timestamp,
            block_number: header.number(),
            block_hash: format!("{:#x}", header.hash()),
        };
        retry_send(&self.query_sender, point.insert_query()).await;
    }
}
