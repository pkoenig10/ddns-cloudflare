use anyhow::{Context, Result};
use reqwest::{Method, StatusCode};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::error;
use std::fmt;
use url::Url;

pub struct Client {
    client: reqwest::blocking::Client,
    token: String,
}

impl Client {
    pub fn new(token: String) -> Client {
        Client {
            client: reqwest::blocking::Client::new(),
            token,
        }
    }

    pub fn zones(&self, name: &str) -> Result<Vec<Zone>> {
        #[derive(Debug, Serialize)]
        struct ListZonesQuery<'a> {
            pub name: &'a str,
        }

        self.request(
            Method::GET,
            "zones",
            Some(ListZonesQuery { name }),
            None::<()>,
        )
    }

    pub fn dns_records(&self, zone_identifier: &str, name: &str) -> Result<Vec<DnsRecord>> {
        #[derive(Debug, Serialize)]
        struct ListDnsRecordsQuery<'a> {
            pub name: &'a str,
        }

        self.request(
            Method::GET,
            &format!("zones/{zone_identifier}/dns_records"),
            Some(ListDnsRecordsQuery { name }),
            None::<()>,
        )
    }

    pub fn patch_dns_record(
        &self,
        zone_identifier: &str,
        identifier: &str,
        content: &str,
    ) -> Result<DnsRecord> {
        #[derive(Debug, Serialize)]
        struct PatchDnsRecordBody<'a> {
            pub content: &'a str,
        }

        self.request(
            Method::PATCH,
            &format!("zones/{zone_identifier}/dns_records/{identifier}"),
            None::<()>,
            Some(PatchDnsRecordBody { content }),
        )
    }

    fn request<ResultType, QueryType, BodyType>(
        &self,
        method: Method,
        path: &str,
        query: Option<QueryType>,
        body: Option<BodyType>,
    ) -> Result<ResultType>
    where
        ResultType: DeserializeOwned,
        QueryType: Serialize,
        BodyType: Serialize,
    {
        let mut request = self
            .client
            .request(method, Self::url(path))
            .bearer_auth(&self.token);
        if let Some(query) = query {
            request = request.query(&query);
        }
        if let Some(body) = body {
            request = request.json(&body);
        }

        let response = request.send()?;

        let status = response.status();
        let body: ResponseBody<ResultType> = response.json().with_context(|| Errors {
            status,
            errors: Vec::new(),
        })?;

        body.result.context(Errors {
            status,
            errors: body.errors,
        })
    }

    fn url(path: &str) -> Url {
        Url::parse("https://api.cloudflare.com/client/v4/")
            .unwrap()
            .join(path)
            .unwrap()
    }
}

#[derive(Debug, Deserialize)]
pub struct Zone {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct DnsRecord {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
struct ResponseBody<ResultType> {
    pub result: Option<ResultType>,
    pub errors: Vec<Error>,
}

#[derive(Debug, Deserialize)]
struct Error {
    pub code: u16,
    pub message: String,
}

#[derive(Debug)]
struct Errors {
    status: StatusCode,
    errors: Vec<Error>,
}

impl fmt::Display for Errors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "HTTP {}", self.status)?;
        for error in &self.errors {
            write!(f, "\n{}: {}", error.code, error.message)?;
        }
        Ok(())
    }
}

impl error::Error for Errors {}
