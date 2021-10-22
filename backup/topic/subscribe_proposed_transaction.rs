use crate::config::Config;
use crate::subscribe::{Subscription, Topic};
use crate::util::retry_send;
use ckb_suite_rpc::ckb_jsonrpc_types::PoolTransactionEntry;
use jsonrpc_core::serde_from_str;
use std::time::Duration;

pub(crate) struct SubscribeProposedTransaction {
    config: Config,
    subscriber: crossbeam::channel::Receiver<(Topic, String)>,
    query_sender: crossbeam::channel::Sender<String>,
    hostname: String,
}

impl SubscribeProposedTransaction {
    pub(crate) fn new(
        config: Config,
        query_sender: crossbeam::channel::Sender<String>,
    ) -> (Self, Subscription) {
        let hostname = gethostname::gethostname().to_string_lossy().to_string();
        let (subscription, subscriber) =
            Subscription::new(config.subscription_url(), Topic::ProposedTransaction);
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
                    assert_eq!(Topic::ProposedTransaction, topic);
                    let pool_transaction_entry =
                        serde_from_str::<PoolTransactionEntry>(&message).unwrap();
                    self.report_new_transaction(&pool_transaction_entry).await;
                }
                Err(crossbeam::channel::TryRecvError::Disconnected) => return,
                Err(crossbeam::channel::TryRecvError::Empty) => {
                    tokio::time::sleep(Duration::from_secs(1)).await
                }
            }
        }
    }
    async fn report_new_transaction(&mut self, pte: &PoolTransactionEntry) {
        let time = chrono::Utc::now().naive_utc();
        let point = crate::table::SubscribeProposedTransaction {
            network: self.config.network(),
            time,
            hostname: self.hostname.clone(),
            transaction_hash: format!("{:#x}", pte.transaction.hash),
        };
        retry_send(&self.query_sender, point.insert_query()).await;
    }
}
