use alloy_primitives::Bytes;
use jsonrpsee::{
    core::{client::ClientT, traits::ToRpcParams},
    http_client::{HttpClient, HttpClientBuilder},
    tracing::{info, trace, warn},
};
use serde::Serialize;
use serde_json::{value::RawValue, Value};
use strum::{Display, EnumIter};

#[derive(Debug, thiserror::Error)]
pub enum BuilderError {
    #[error("HTTP client error: {0}")]
    ClientError(#[from] jsonrpsee::core::ClientError),
    #[error("Invalid response from builder: {0}")]
    InvalidResponse(Value),
}

pub struct BuilderEndpoint {
    url: String,
    rpc_method: String,
}

impl BuilderEndpoint {
    fn new(url: &str, rpc_method: &str) -> Self {
        Self {
            url: url.to_string(),
            rpc_method: rpc_method.to_string(),
        }
    }
}

#[derive(Clone, EnumIter, Display)]
pub enum BuilderKind {
    Titan,
    Beaver,
    Rsync,
}

impl BuilderKind {
    fn endpoint(&self) -> BuilderEndpoint {
        match self {
            BuilderKind::Titan => {
                BuilderEndpoint::new("https://rpc.titanbuilder.xyz", "eth_sendPrivateTransaction")
            }
            BuilderKind::Beaver => BuilderEndpoint::new(
                "https://mevshare-rpc.beaverbuild.org",
                "eth_sendPrivateRawTransaction",
            ),
            BuilderKind::Rsync => {
                BuilderEndpoint::new("https://rsync-builder.xyz", "eth_sendPrivateRawTransaction")
            }
        }
    }

    pub fn builder(&self) -> Result<Builder, BuilderError> {
        let endpoint = self.endpoint();
        let client = HttpClientBuilder::default()
            .max_request_size(10 * 1024 * 1024) // 10MB
            .build(endpoint.url)?;

        Ok(Builder {
            client,
            rpc_method: endpoint.rpc_method,
            kind: self.clone(),
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TxRequest {
    pub tx: Bytes,
}

impl ToRpcParams for TxRequest {
    fn to_rpc_params(self) -> Result<Option<Box<RawValue>>, serde_json::Error> {
        let json_value = serde_json::to_value(&self)?;
        let raw_value = RawValue::from_string(json_value.to_string())?;
        Ok(Some(raw_value))
    }
}

pub struct Builder {
    client: HttpClient,
    rpc_method: String,
    kind: BuilderKind,
}

impl Builder {
    pub async fn send_tx(&self, tx: Bytes) -> Result<(), BuilderError> {
        let params = TxRequest { tx };
        trace!(target: "builder", ?params, "Sending tx to builder: {}", self.kind);
        let response: Value = self.client.request(&self.rpc_method, params).await?;
        if response.get("error").is_some() {
            warn!(target: "builder", ?response, "Builder returned error response: {}", self.kind);
            return Err(BuilderError::InvalidResponse(response));
        }
        info!(target: "builder", ?response, "Tx successfully submitted to builder: {}", self.kind);
        Ok(())
    }
}
