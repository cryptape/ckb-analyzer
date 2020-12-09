use ckb_jsonrpc_types::{
    Block, BlockNumber, BlockTemplate, BlockView, CellOutputWithOutPoint, CellWithStatus,
    ChainInfo, DryRunResult, EpochNumber, EpochView, HeaderView, LocalNode, OutPoint, PeerState,
    RemoteNode, Transaction, TransactionWithStatus, TxPoolInfo, Uint64, Version,
};
use ckb_types::{
    core::{
        BlockNumber as CoreBlockNumber, EpochNumber as CoreEpochNumber, Version as CoreVersion,
    },
    packed::Byte32,
    prelude::*,
    H256,
};
use ckb_util::Mutex;
use hyper::header::{Authorization, Basic};
use jsonrpc_client_core::{expand_params, jsonrpc_client, Result as JsonRpcResult};
use jsonrpc_client_http::{HttpHandle, HttpTransport};
use std::env::var;
use std::sync::Arc;

#[derive(Clone)]
pub struct Jsonrpc {
    uri: String,
    inner: Arc<Mutex<Inner<HttpHandle>>>,
}

pub fn env_username() -> String {
    var("CKB_RPC_USERNAME").unwrap_or_else(|_| "".to_owned())
}

pub fn env_password() -> String {
    var("CKB_RPC_PASSWORD").unwrap_or_else(|_| "".to_owned())
}

impl Jsonrpc {
    pub fn connect(uri: &str) -> Self {
        let transport = HttpTransport::new().standalone().unwrap();
        let mut transport = transport
            .handle(uri)
            .unwrap_or_else(|err| panic!("Jsonrpc::connect(\"{}\"), error: {:?}", uri, err));

        if !env_username().is_empty() && !env_password().is_empty() {
            transport.set_header(Authorization(Basic {
                username: env_username(),
                password: Some(env_password()),
            }));
        }
        Self {
            uri: uri.to_string(),
            inner: Arc::new(Mutex::new(Inner::new(transport))),
        }
    }

    pub fn uri(&self) -> &String {
        &self.uri
    }

    pub fn inner(&self) -> &Mutex<Inner<HttpHandle>> {
        &self.inner
    }

    pub fn get_block(&self, hash: Byte32) -> Option<BlockView> {
        self.inner
            .lock()
            .get_block(hash.unpack())
            .call()
            .unwrap_or_else(|err| {
                panic!(
                    "Jsonrpc::get_block(\"{}\", {}), error: {:?}",
                    self.uri(),
                    hash,
                    err
                )
            })
    }

    pub fn get_header(&self, hash: Byte32) -> Option<HeaderView> {
        self.inner
            .lock()
            .get_header(hash.unpack())
            .call()
            .unwrap_or_else(|err| {
                panic!(
                    "Jsonrpc::get_header(\"{}\", {}), error: {:?}",
                    self.uri(),
                    hash,
                    err
                )
            })
    }

    pub fn get_block_by_number(&self, number: CoreBlockNumber) -> Option<BlockView> {
        self.inner
            .lock()
            .get_block_by_number(number.into())
            .call()
            .unwrap_or_else(|err| {
                panic!(
                    "Jsonrpc::get_block_by_number(\"{}\", {}), error: {:?}",
                    self.uri(),
                    number,
                    err
                )
            })
    }

    pub fn get_transaction(&self, hash: Byte32) -> Option<TransactionWithStatus> {
        self.inner
            .lock()
            .get_transaction(hash.unpack())
            .call()
            .unwrap_or_else(|err| {
                panic!(
                    "Jsonrpc::get_transaction(\"{}\", {}), error: {:?}",
                    self.uri(),
                    hash,
                    err
                )
            })
    }

    pub fn get_block_hash(&self, number: CoreBlockNumber) -> Option<H256> {
        self.inner
            .lock()
            .get_block_hash(number.into())
            .call()
            .unwrap_or_else(|err| {
                panic!(
                    "Jsonrpc::get_block_hash(\"{}\", {}), error: {:?}",
                    self.uri(),
                    number,
                    err
                )
            })
    }

    pub fn get_tip_header(&self) -> HeaderView {
        self.inner
            .lock()
            .get_tip_header()
            .call()
            .unwrap_or_else(|err| {
                panic!(
                    "Jsonrpc::get_tip_header(\"{}\"), error: {:?}",
                    self.uri(),
                    err
                )
            })
    }

    pub fn get_header_by_number(&self, number: CoreBlockNumber) -> Option<HeaderView> {
        self.inner
            .lock()
            .get_header_by_number(number.into())
            .call()
            .unwrap_or_else(|err| {
                panic!(
                    "Jsonrpc::get_header_by_number(\"{}\", {}), error: {:?}",
                    self.uri(),
                    number,
                    err
                )
            })
    }

    pub fn get_cells_by_lock_hash(
        &self,
        lock_hash: Byte32,
        from: CoreBlockNumber,
        to: CoreBlockNumber,
    ) -> Vec<CellOutputWithOutPoint> {
        self.inner
            .lock()
            .get_cells_by_lock_hash(lock_hash.unpack(), from.into(), to.into())
            .call()
            .unwrap_or_else(|err| {
                panic!(
                    "Jsonrpc::get_cells_by_lock_hash(\"{}\", {}, {}, {}), error: {:?}",
                    self.uri(),
                    lock_hash,
                    from,
                    to,
                    err
                )
            })
    }

    pub fn get_live_cell(&self, out_point: OutPoint) -> CellWithStatus {
        self.inner
            .lock()
            .get_live_cell(out_point.clone())
            .call()
            .unwrap_or_else(|err| {
                panic!(
                    "Jsonrpc::get_live_cell(\"{}\", {:?}), error: {:?}",
                    self.uri(),
                    out_point,
                    err
                )
            })
    }

    pub fn get_tip_block_number(&self) -> CoreBlockNumber {
        self.inner
            .lock()
            .get_tip_block_number()
            .call()
            .unwrap_or_else(|err| {
                panic!(
                    "Jsonrpc::get_tip_block_number(\"{}\"), error: {:?}",
                    self.uri(),
                    err
                )
            })
            .into()
    }

    pub fn local_node_info(&self) -> LocalNode {
        self.inner
            .lock()
            .local_node_info()
            .call()
            .unwrap_or_else(|err| {
                panic!(
                    "Jsonrpc::local_node_info(\"{}\"), error: {:?}",
                    self.uri(),
                    err
                )
            })
    }

    pub fn get_peers(&self) -> Vec<RemoteNode> {
        self.inner.lock().get_peers().call().unwrap_or_else(|err| {
            panic!("Jsonrpc::get_peers(\"{}\"), error: {:?}", self.uri(), err)
        })
    }

    pub fn get_block_template(
        &self,
        bytes_limit: Option<u64>,
        proposals_limit: Option<u64>,
        max_version: Option<CoreVersion>,
    ) -> BlockTemplate {
        let bytes_limit = bytes_limit.map(Into::into);
        let proposals_limit = proposals_limit.map(Into::into);
        let max_version = max_version.map(Into::into);
        self.inner
            .lock()
            .get_block_template(bytes_limit, proposals_limit, max_version)
            .call()
            .unwrap_or_else(|err| {
                panic!(
                    "Jsonrpc::get_block_template(\"{}\", {:?}, {:?}, {:?}), error: {:?}",
                    self.uri(),
                    bytes_limit,
                    proposals_limit,
                    max_version,
                    err
                )
            })
    }

    pub fn submit_block(&self, work_id: String, block: Block) -> Option<H256> {
        self.inner
            .lock()
            .submit_block(work_id, block.clone())
            .call()
            .unwrap_or_else(|err| {
                panic!(
                    "Jsonrpc::submit_block(\"{}\", {:?}), error: {:?}",
                    self.uri(),
                    block,
                    err
                )
            })
    }

    pub fn get_blockchain_info(&self) -> ChainInfo {
        self.inner
            .lock()
            .get_blockchain_info()
            .call()
            .unwrap_or_else(|err| {
                panic!(
                    "Jsonrpc::get_blockchain_info(\"{}\"), error: {:?}",
                    self.uri(),
                    err
                )
            })
    }

    pub fn send_transaction(&self, tx: Transaction) -> H256 {
        self.inner
            .lock()
            .send_transaction(tx.clone())
            .call()
            .unwrap_or_else(|err| {
                panic!(
                    "Jsonrpc::send_transaction(\"{}\", {:?}), error: {:?}",
                    self.uri(),
                    tx,
                    err
                )
            })
    }

    pub fn broadcast_transaction(&self, tx: Transaction) -> H256 {
        self.inner
            .lock()
            .broadcast_transaction(tx.clone())
            .call()
            .unwrap_or_else(|err| {
                panic!(
                    "Jsonrpc::broadcast_transaction(\"{}\", {:?}), error: {:?}",
                    self.uri(),
                    tx,
                    err
                )
            })
    }

    pub fn send_transaction_result(&self, tx: Transaction) -> JsonRpcResult<H256> {
        self.inner.lock().send_transaction(tx).call()
    }

    pub fn tx_pool_info(&self) -> TxPoolInfo {
        self.inner
            .lock()
            .tx_pool_info()
            .call()
            .unwrap_or_else(|err| {
                panic!(
                    "Jsonrpc::tx_pool_info(\"{}\"), error: {:?}",
                    self.uri(),
                    err
                )
            })
    }

    pub fn get_current_epoch(&self) -> EpochView {
        self.inner
            .lock()
            .get_current_epoch()
            .call()
            .unwrap_or_else(|err| {
                panic!(
                    "Jsonrpc::get_current_epoch(\"{}\"), error: {:?}",
                    self.uri(),
                    err
                )
            })
    }

    pub fn get_epoch_by_number(&self, number: CoreEpochNumber) -> Option<EpochView> {
        self.inner
            .lock()
            .get_epoch_by_number(number.into())
            .call()
            .unwrap_or_else(|err| {
                panic!(
                    "Jsonrpc::get_epoch_by_number(\"{}\", {}), error: {:?}",
                    self.uri(),
                    number,
                    err
                )
            })
    }

    pub fn get_fork_block(&self, hash: Byte32) -> Option<BlockView> {
        self.inner
            .lock()
            .get_fork_block(hash.unpack())
            .call()
            .unwrap_or_else(|err| {
                panic!(
                    "Jsonrpc::get_fork_block(\"{}\", {}), error: {:?}",
                    self.uri(),
                    hash,
                    err
                )
            })
    }

    pub fn add_node(&self, peer_id: String, address: String) {
        self.inner
            .lock()
            .add_node(peer_id.clone(), address.clone())
            .call()
            .unwrap_or_else(|err| {
                panic!(
                    "Jsonrpc::add_node(\"{}\", {}, {}), error: {:?}",
                    self.uri(),
                    peer_id,
                    address,
                    err,
                )
            });
    }

    pub fn remove_node(&self, peer_id: String) {
        self.inner
            .lock()
            .remove_node(peer_id.clone())
            .call()
            .unwrap_or_else(|err| {
                panic!(
                    "Jsonrpc::remove_node(\"{}\", {}), error: {:?}",
                    self.uri(),
                    peer_id,
                    err
                )
            })
    }

    pub fn process_block_without_verify(&self, block: Block) -> Option<H256> {
        self.inner
            .lock()
            .process_block_without_verify(block.clone())
            .call()
            .unwrap_or_else(|err| {
                panic!(
                    "Jsonrpc::process_block_without_verify(\"{}\", {:?}), error: {:?}",
                    self.uri(),
                    block,
                    err
                )
            })
    }
}

jsonrpc_client!(pub struct Inner {
    pub fn get_block(&mut self, _hash: H256) -> RpcRequest<Option<BlockView>>;
    pub fn get_header(&mut self, _hash: H256) -> RpcRequest<Option<HeaderView>>;
    pub fn get_block_by_number(&mut self, _number: BlockNumber) -> RpcRequest<Option<BlockView>>;
    pub fn get_header_by_number(&mut self, _number: BlockNumber) -> RpcRequest<Option<HeaderView>>;
    pub fn get_transaction(&mut self, _hash: H256) -> RpcRequest<Option<TransactionWithStatus>>;
    pub fn get_block_hash(&mut self, _number: BlockNumber) -> RpcRequest<Option<H256>>;
    pub fn get_tip_header(&mut self) -> RpcRequest<HeaderView>;
    pub fn get_cells_by_lock_hash(
        &mut self,
        _lock_hash: H256,
        _from: BlockNumber,
        _to: BlockNumber
    ) -> RpcRequest<Vec<CellOutputWithOutPoint>>;
    pub fn get_live_cell(&mut self, _out_point: OutPoint) -> RpcRequest<CellWithStatus>;
    pub fn get_tip_block_number(&mut self) -> RpcRequest<BlockNumber>;
    pub fn local_node_info(&mut self) -> RpcRequest<LocalNode>;
    pub fn get_peers(&mut self) -> RpcRequest<Vec<RemoteNode>>;
    pub fn get_block_template(
        &mut self,
        bytes_limit: Option<Uint64>,
        proposals_limit: Option<Uint64>,
        max_version: Option<Version>
    ) -> RpcRequest<BlockTemplate>;
    pub fn submit_block(&mut self, _work_id: String, _data: Block) -> RpcRequest<Option<H256>>;
    pub fn get_blockchain_info(&mut self) -> RpcRequest<ChainInfo>;
    pub fn get_peers_state(&mut self) -> RpcRequest<Vec<PeerState>>;
    pub fn compute_transaction_hash(&mut self, tx: Transaction) -> RpcRequest<H256>;
    pub fn dry_run_transaction(&mut self, _tx: Transaction) -> RpcRequest<DryRunResult>;
    pub fn send_transaction(&mut self, tx: Transaction) -> RpcRequest<H256>;
    pub fn broadcast_transaction(&mut self, tx: Transaction) -> RpcRequest<H256>;
    pub fn tx_pool_info(&mut self) -> RpcRequest<TxPoolInfo>;
    pub fn get_current_epoch(&mut self) -> RpcRequest<EpochView>;
    pub fn get_epoch_by_number(&mut self, number: EpochNumber) -> RpcRequest<Option<EpochView>>;
    pub fn get_fork_block(&mut self, _hash: H256) -> RpcRequest<Option<BlockView>>;

    pub fn add_node(&mut self, peer_id: String, address: String) -> RpcRequest<()>;
    pub fn remove_node(&mut self, peer_id: String) -> RpcRequest<()>;
    pub fn process_block_without_verify(&mut self, _data: Block) -> RpcRequest<Option<H256>>;
});
