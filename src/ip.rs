use anyhow::{anyhow, Context, Result};
use std::error;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::str::{from_utf8, FromStr};
use trust_dns_client::client::{Client, SyncClient};
use trust_dns_client::rr::{DNSClass, RecordType};
use trust_dns_client::udp::UdpClientConnection;

pub trait Ip: FromStr {
    fn name_server() -> IpAddr;
}

impl Ip for Ipv4Addr {
    fn name_server() -> IpAddr {
        "1.1.1.1".parse().unwrap()
    }
}

impl Ip for Ipv6Addr {
    fn name_server() -> IpAddr {
        "2606:4700:4700::1111".parse().unwrap()
    }
}

pub fn query<I>() -> Result<I>
where
    I: Ip,
    <I as FromStr>::Err: error::Error + Send + Sync + 'static,
{
    let connection = UdpClientConnection::new(SocketAddr::new(I::name_server(), 53))
        .context("Failed to create connection")?;
    let client = SyncClient::new(connection);

    let response = client
        .query(
            &"whoami.cloudflare".parse().unwrap(),
            DNSClass::CH,
            RecordType::TXT,
        )
        .context("Failed to execute query")?;

    let data = response
        .answers()
        .first()
        .ok_or(anyhow!("No answers"))?
        .rdata()
        .as_txt()
        .ok_or(anyhow!("Invalid record type"))?
        .txt_data()
        .first()
        .ok_or(anyhow!("No record data"))?;

    from_utf8(data)
        .context("Invalid record data")?
        .parse()
        .context("Failed to parse record data")
}
