use crate::CONFIG;
use ckb_app_config::CKBAppConfig;

pub(crate) fn app_config() -> CKBAppConfig {
    match CONFIG.network.ckb_network_name.as_str() {
        "lina" => {
            let bytes = include_bytes!("../ckb_app_config/lina.toml");
            toml::from_slice(bytes).unwrap()
        }
        "aggron" => {
            let bytes = include_bytes!("../ckb_app_config/aggron.toml");
            toml::from_slice(bytes).unwrap()
        }
        _ => unimplemented!(),
    }
}
