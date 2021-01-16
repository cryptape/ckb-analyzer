use ckb_suite_rpc::Jsonrpc;

pub fn get_network_identifier(ckb_rpc_url: &str) -> String {
    let jsonrpc = Jsonrpc::connect(ckb_rpc_url);
    let consensus = jsonrpc.get_consensus();
    let genesis_hash = format!("{:x}", consensus.genesis_hash);
    format!("/{}/{}", consensus.id, &genesis_hash[..8])
}
