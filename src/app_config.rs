use crate::CKB_NETWORK;
use ckb_app_config::CKBAppConfig;

pub(crate) fn app_config() -> CKBAppConfig {
    match CKB_NETWORK.as_str() {
        "mainnet" => {
            let bytes = include_bytes!("../specs/ckb.mainnet.toml");
            toml::from_slice(bytes).unwrap()
        }
        "testnet" => {
            let bytes = include_bytes!("../specs/ckb.testnet.toml");
            toml::from_slice(bytes).unwrap()
        }
        _ => unimplemented!(),
    }
}
