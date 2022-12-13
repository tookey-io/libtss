use crate::console_log;
use crate::wasm::log;
use anyhow::{Context, Result};
use futures::{Sink, SinkExt, Stream, StreamExt, TryStreamExt};
use round_based::Msg;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::io::{Error, ErrorKind};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};

pub async fn join_computation<M>(
  address: String,
  room_id: &str,
) -> Result<(
  u16,
  impl Stream<Item = Result<Msg<M>>>,
  impl Sink<Msg<M>, Error = anyhow::Error>,
)>
where
  M: Serialize + DeserializeOwned,
{
  let client = SmClient::new(address, room_id);

  // Construct channel of incoming messages
  let incoming = client
    .subscribe()
    .await
    .context("subscribe")?
    .and_then(|msg| async move { serde_json::from_str::<Msg<M>>(&msg).context("deserialize message") });

  // Obtain party index
  let index = client.issue_index().await.context("issue an index")?;

  // Ignore incoming messages addressed to someone else
  let incoming = incoming.try_filter(move |msg| {
    futures::future::ready(msg.sender != index && (msg.receiver.is_none() || msg.receiver == Some(index)))
  });

  // Construct channel of outgoing messages
  let outgoing = futures::sink::unfold(client, |client, message: Msg<M>| async move {
    let serialized = serde_json::to_string(&message).context("serialize message")?;
    client.broadcast(serialized).await.context("broadcast message")?;
    Ok::<_, anyhow::Error>(client)
  });

  Ok((index, incoming, outgoing))
}

pub struct SmClient {
  base_url: String,
}

impl SmClient {
  pub fn new(address: String, room_id: &str) -> Self {
    Self {
      base_url: format!("{address}/rooms/{room_id}"),
    }
  }

  pub async fn issue_index(&self) -> Result<u16> {
    let response = reqwest::Client::new()
      .post(format!("{}/issue_unique_idx", self.base_url))
      .send()
      .await?
      .text()
      .await?;

    let response: IssuedUniqueIdx = serde_json::from_str(&response)?;
    Ok(response.unique_idx)
  }

  pub async fn broadcast(&self, message: String) -> Result<()> {
    reqwest::Client::new()
      .post(format!("{}/broadcast", self.base_url))
      .body(message)
      .send()
      .await?;

    Ok(())
  }

  pub async fn subscribe(&self) -> Result<impl Stream<Item = Result<String>>> {
    let (mut sender, receiver) = futures::channel::mpsc::unbounded::<Result<String>>();

    let client = web_sys::EventSource::new(&format!("{}/subscribe", self.base_url))
      .map_err(|e| anyhow::anyhow!("JsError: {:?}", e))
      .unwrap();

    let cb: Closure<dyn FnMut(JsValue)> = Closure::new(move |message: JsValue| {
      console_log!("Received message: {:?}", message);
      let message: web_sys::MessageEvent = message.into();

      let _ = sender.send(message.data().as_string().context("SSE data is not string"));
    });

    client.set_onmessage(Some(&cb.as_ref().unchecked_ref()));

    let stream = receiver
      .into_stream()
      .map_err(|e| Error::new(ErrorKind::Other, e))
      .into_async_read();
    let events = async_sse::decode(stream);

    Ok(events.filter_map(|msg| async {
      match msg {
        Ok(async_sse::Event::Message(msg)) => {
          Some(String::from_utf8(msg.into_bytes()).context("SSE message is not valid UTF-8 string"))
        }
        Ok(_) => {
          // ignore other types of events
          None
        }
        Err(e) => Some(Err(e.into_inner())),
      }
    }))
  }
}

#[derive(Deserialize, Debug)]
pub struct IssuedUniqueIdx {
  unique_idx: u16,
}
