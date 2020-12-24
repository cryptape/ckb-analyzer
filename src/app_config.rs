use ckb_app_config::CKBAppConfig;

pub(crate) fn app_config(ckb_network_name: &str) -> CKBAppConfig {
    match ckb_network_name {
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
