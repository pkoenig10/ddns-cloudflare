use anyhow::{Context, Result};
use log::log;
use std::env::{self, VarError};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::process::ExitCode;

mod cloudflare;
mod ip;
mod log;

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            log!("{:#}", e);
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<()> {
    let cloudflare_api_token = env_var("CLOUDFLARE_API_TOKEN", None)?;
    let domain = env_var("DOMAIN", None)?;

    let client = cloudflare::Client::new(cloudflare_api_token);

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
            log!(
                "No update required for {} record ({})",
                dns_record.type_,
                dns_record.content
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

        log!(
            "Updated {} record from {} to {}",
            dns_record.type_,
            dns_record.content,
            ip_address
        );
    }

    Ok(())
}

fn env_var(name: &str, default: Option<&str>) -> Result<String> {
    let value = env::var(name)
        .or_else(|err| {
            if let VarError::NotPresent = err
                && let Some(default) = default
            {
                Ok(default.to_string())
            } else {
                Err(err)
            }
        })
        .with_context(|| format!("Environment variable {} is required but not set", name))?;

    Ok(value)
}
