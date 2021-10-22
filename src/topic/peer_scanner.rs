use crate::tokio;
use std::env;
use std::time::Duration;

pub struct PeerScanner {
    pg: tokio_postgres::Client,
    ipinfo: ipinfo::IpInfo,
}

impl PeerScanner {
    pub async fn new(pg_config: &tokio_postgres::Config) -> Self {
        let (pg, pg_connection) = pg_config.connect(tokio_postgres::NoTls).await.unwrap();
        tokio::spawn(async move {
            if let Err(err) = pg_connection.await {
                log::error!("postgres connection error: {:?}", err);
            }
        });

        let ipinfo_io_token = match env::var("IPINFO_IO_TOKEN") {
            Ok(token) if !token.is_empty() => Some(token),
            _ => {
                log::warn!("miss environment variable \"IPINFO_IO_TOKEN\", use empty value");
                None
            }
        };
        let ipinfo = ipinfo::IpInfo::new(ipinfo::IpInfoConfig {
            token: ipinfo_io_token,
            cache_size: 1000,
            timeout: ::std::time::Duration::from_secs(2 * 60),
        })
        .expect("connect to https://ipinfo.io");

        Self { pg, ipinfo }
    }

    pub async fn run(&mut self) {
        loop {
            if let Err(err) = self.run_().await {
                log::error!("postgres error: {:?}", err);
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
        }
    }

    async fn run_(&mut self) -> Result<(), tokio_postgres::Error> {
        let statement = self
            .pg
            .prepare("SELECT id, network, time, version, ip FROM peer WHERE id > $1 AND country IS NULL ORDER BY ID LIMIT 100")
            .await?;
        let mut last_id = 0i32;

        loop {
            let raws = self.pg.query(&statement, &[&last_id]).await?;
            if raws.is_empty() {
                log::debug!("select null-country peer, empty results");
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }

            last_id = raws[raws.len() - 1].get(0);
            log::debug!("select null-country peer, last_id: {}", last_id);
            for raw in raws {
                let id: i32 = raw.get(0);
                let ip: String = raw.get(4);

                match self.lookup_country(&ip) {
                    Ok(country) => {
                        let raw_query =
                            format!("UPDATE peer SET country = '{}' WHERE id = {}", country, id,);
                        self.pg.batch_execute(&raw_query).await?;
                    }
                    Err(err) => {
                        log::error!("ipinfo.io query error: {:?}", err);
                        continue;
                    }
                }
            }
        }
    }

    pub fn lookup_country(&mut self, ip: &str) -> Result<String, ipinfo::IpError> {
        let info_map = self.ipinfo.lookup(&[ip])?;
        let ipdetail = info_map[ip].to_owned();
        Ok(ipdetail.country)
    }
}
