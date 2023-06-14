use std::time::Duration;

use crate::ceremonies;
use anyhow::{anyhow, Context};
use curv::arithmetic::Converter;
use curv::BigInt;
use futures::{SinkExt, StreamExt};
use napi_derive::napi;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::ecdsa::state_machine::sign::{OfflineProtocolMessage, PartialSignature};
use crate::join::join_computation;

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignParams {
  pub room_id: String,
  pub key: String,
  pub data: String,
  pub participants_indexes: Vec<u16>,
  pub relay_address: String,
  pub timeout_seconds: u16,
}

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignResult {
  pub result: Option<String>,
  pub error: Option<String>,
}

#[allow(dead_code)]
pub async fn sign(params: SignParams) -> SignResult {
  match internal_sign(params).await {
    Ok(val) => SignResult {
      result: Some(val),
      error: None,
    },
    Err(err) => SignResult {
      result: None,
      error: Some(format!("{:?}", err)),
    },
  }
}

async fn internal_sign(params: SignParams) -> anyhow::Result<String> {
  let (_i, incoming, outgoing) = join_computation::<Value>(params.relay_address, params.room_id.as_str())
    .await
    .context("join computation")?;

  let key = serde_json::from_str(&params.key)?;

  // Proceed with protocol
  let incoming = incoming.filter_map(|msg| async move {
    match msg {
      Ok(msg) => {
        // TODO: fix coping
        let json = msg.body.clone();
        let possible_offline = serde_json::from_value::<OfflineProtocolMessage>(json.clone());
        let possible_partial = serde_json::from_value::<PartialSignature>(json);

        let wrapped_msg = match (possible_offline, possible_partial) {
          (Ok(offline), _) => ceremonies::Messages::OfflineStage(msg.map_body(|_| offline)),
          (_, Ok(partial)) => ceremonies::Messages::PartialSignature(msg.map_body(|_| partial)),
          _ => unreachable!(),
        };

        Some(Ok(wrapped_msg))
      }
      Err(e) => Some(Err(e)),
    }
  });

  tokio::pin!(outgoing);

  let outgoing_sender = futures::sink::unfold(outgoing, |mut outgoing, msg| async move {
    let packet = match msg {
      ceremonies::Messages::OfflineStage(msg) => msg.map_body(|body| serde_json::to_value(body).unwrap_or(Value::Null)),
      ceremonies::Messages::PartialSignature(msg) => {
        msg.map_body(|body| serde_json::to_value(body).unwrap_or(Value::Null))
      }
    };

    outgoing.send(packet).await.context("sending")?;

    Ok::<_, anyhow::Error>(outgoing)
  });

  let hash = parse_hash(params.data)?;

  let signature_future = ceremonies::sign(
    outgoing_sender,
    incoming,
    key,
    params.participants_indexes.clone(),
    hash,
  );

  match tokio::time::timeout(Duration::from_secs(params.timeout_seconds as u64), signature_future).await {
    Ok(result) => match result {
      Ok(val) => serde_json::to_string(&val).context("serialize signature"),
      Err(err) => Err(anyhow!("{:?}", err)),
    },
    Err(_) => Err(anyhow!("Timed out")),
  }
}

fn parse_hash(str: String) -> anyhow::Result<BigInt> {
  let str = str.strip_prefix("0x").unwrap_or(&str);
  let bytes = hex::decode(str)?;

  if bytes.len() == 32 {
    Ok(BigInt::from_bytes(bytes.as_slice()))
  } else {
    Err(anyhow!("Hash has incorrect length: expected 32, got {}", bytes.len()))
  }
}
