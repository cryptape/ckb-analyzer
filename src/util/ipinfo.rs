use ipinfo::{IpDetails, IpError, IpInfo};
use lazy_static::lazy_static;
use std::sync::RwLock;

lazy_static! {
    static ref IPINFO: RwLock<IpInfo> = {
        let ipinfo_io_token = match ::std::env::var("IPINFO_IO_TOKEN") {
            Ok(token) if !token.is_empty() => Some(token),
            _ => {
                log::warn!("miss environment variable \"IPINFO_IO_TOKEN\", use empty value");
                None
            }
        };
        let ipinfo = ipinfo::IpInfo::new(ipinfo::IpInfoConfig {
            token: ipinfo_io_token,
            cache_size: 10000,
            timeout: ::std::time::Duration::from_secs(365 * 24 * 60 * 60),
        })
        .expect("connect to https://ipinfo.io");
        RwLock::new(ipinfo)
    };
}

pub fn lookup_ipinfo(ip: &str) -> Result<IpDetails, IpError> {
    let infos = IPINFO.write().unwrap().lookup(&[ip])?;
    Ok(infos[ip].to_owned())
}
