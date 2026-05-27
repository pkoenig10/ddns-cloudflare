use anyhow::{Context, Result, bail};
use hickory_net::client::{Client, ClientHandle};
use hickory_net::proto::rr::{DNSClass, RData::TXT, RecordType};
use hickory_net::runtime::TokioRuntimeProvider;
use hickory_net::udp::UdpClientStream;
use std::error;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::str::{FromStr, from_utf8};

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

pub async fn query<I>() -> Result<I>
where
    I: Ip,
    <I as FromStr>::Err: error::Error + Send + Sync + 'static,
{
    let connection = UdpClientStream::builder(
        SocketAddr::new(I::name_server(), 53),
        TokioRuntimeProvider::default(),
    )
    .build();
    let (mut client, background) = Client::<TokioRuntimeProvider>::from_sender(connection);
    tokio::spawn(background);

    let response = client
        .query(
            "whoami.cloudflare".parse().unwrap(),
            DNSClass::CH,
            RecordType::TXT,
        )
        .await
        .context("Failed to execute query")?;

    let record = response.answers.first().context("No answers")?;

    let TXT(txt) = &record.data else {
        bail!("Invalid record type")
    };

    let data = txt.txt_data.first().context("No TXT record data")?;

    from_utf8(data)
        .context("Invalid record data")?
        .parse()
        .context("Failed to parse record data")
}
