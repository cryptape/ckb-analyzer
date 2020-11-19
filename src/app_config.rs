use crate::CONFIG;
use ckb_app_config::CKBAppConfig;

pub(crate) fn app_config() -> CKBAppConfig {
    match CONFIG.network.ckb_app_config.as_str() {
        "ckb.mainnet.toml" => {
            let bytes = include_bytes!("../specs/ckb.mainnet.toml");
            toml::from_slice(bytes).unwrap()
        }
        "ckb.testnet.toml" => {
            let bytes = include_bytes!("../specs/ckb.testnet.toml");
            toml::from_slice(bytes).unwrap()
        }
        _ => unimplemented!(),
    }
}
