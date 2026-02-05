use ::log::{info, LevelFilter};
use anyhow::{Context, Result};
use std::env;
use std::net::{Ipv4Addr, Ipv6Addr};

mod cloudflare;
mod ip;
mod log;

const API_TOKEN_ENV_VAR: &str = "API_TOKEN";
const DOMAIN_ENV_VAR: &str = "DOMAIN";

#[tokio::main]
async fn main() -> Result<()> {
    log::init(LevelFilter::Info).context("Failed to initialize logger")?;

    let token = env::var(API_TOKEN_ENV_VAR).context("Failed to fetch API token")?;
    let domain = env::var(DOMAIN_ENV_VAR).context("Failed to fetch domain")?;

    let client = cloudflare::Client::new(token);

    let zone = client
        .zones(&domain)
        .await
        .context("Failed to get zone")?
        .into_iter()
        .next()
        .context("Zone not found")?;

    let dns_records = client
        .dns_records(&zone.id, &domain)
        .await
        .context("Failed to get DNS records")?;

    for dns_record in dns_records {
        let ip_address = match dns_record.type_.as_str() {
            "A" => ip::query::<Ipv4Addr>()
                .await
                .context("Failed to get IPv4 address")?
                .to_string(),
            "AAAA" => ip::query::<Ipv6Addr>()
                .await
                .context("Failed to get IPv6 address")?
                .to_string(),
            _ => continue,
        };

        if dns_record.content == ip_address {
            info!(
                "No update required for {} record ({})",
                dns_record.type_, dns_record.content
            );
            continue;
        }

        client
            .patch_dns_record(&zone.id, &dns_record.id, &ip_address)
            .await
            .with_context(|| {
                format!(
                    "Failed to update {} record from {} to {}",
                    dns_record.type_, dns_record.content, ip_address,
                )
            })?;

        info!(
            "Updated {} record from {} to {}",
            dns_record.type_, dns_record.content, ip_address
        );
    }

    Ok(())
}
