use reqwest::{Method, Response};

use crate::{query::Query, Measurement};

pub struct InfluxClient {
    url: String,
    key: String,
    org: String,
    http_client: reqwest::Client,
}

impl InfluxClient {
    fn new(url: String, key: String, org: String, http_client: reqwest::Client) -> Self {
        Self {
            url,
            key,
            org,
            http_client,
        }
    }

    pub fn builder(url: String, key: String, org: String) -> InfluxClientBuilder {
        InfluxClientBuilder::new(url, key, org)
    }

    pub async fn send_batch(&self, bucket: &str, measurements: &[Measurement]) -> Response {
        let payload = measurements
            .iter()
            .map(|m| m.to_line_protocol())
            .collect::<Vec<_>>()
            .join("\n");
        let url = format!(
            "{}/api/v2/write?org={}&bucket={}&precision=ms",
            self.url, self.org, bucket
        );
        info!("posting payload to influx at '{}': '{}'", url, payload);
        self.http_client
            .post(&url)
            .header("Authorization", format!("Token {}", &self.key))
            .body(payload)
            .send()
            .await
            .unwrap()
    }

    pub async fn send_query(&self, query: Query) -> Result<Response, InfluxClientError> {
        let payload = query.to_string();

        debug!("sending query: {}", payload);

        let request = self
            .http_client
            .request(
                Method::POST,
                &format!("{}/api/v2/query?org={}", self.url, self.org),
            )
            .header("Authorization", format!("Token {}", &self.key))
            .header("Content-type", "application/vnd.flux")
            .header("Accept", "application/csv")
            .body(payload)
            .build()?;

        let response = self.http_client.execute(request).await?;

        Ok(response)
    }
}

pub struct InfluxClientBuilder {
    url: String,
    key: String,
    org: String,
}

impl InfluxClientBuilder {
    fn new(url: String, key: String, org: String) -> Self {
        Self { url, key, org }
    }

    pub fn build(self) -> Result<InfluxClient, InfluxClientBuilderError> {
        Ok(InfluxClient::new(
            self.url,
            self.key,
            self.org,
            reqwest::Client::new(),
        ))
    }
}

#[derive(Debug)]
pub enum InfluxClientBuilderError {}

#[derive(Debug)]
pub enum InfluxClientError {
    ReqwestError(reqwest::Error),
}

impl From<reqwest::Error> for InfluxClientError {
    fn from(err: reqwest::Error) -> Self {
        Self::ReqwestError(err)
    }
}
