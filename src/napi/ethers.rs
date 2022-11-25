use crate::ecdsa::party_i::SignatureRecid;
use crate::ecdsa::state_machine::keygen::LocalKey;
use anyhow::Context;
use curv::arithmetic::Integer;
use curv::elliptic::curves::secp256_k1::Secp256k1Scalar;
use curv::elliptic::curves::{ECScalar, Scalar, Secp256k1};
use curv::BigInt;
use ethereum_types::Signature;
use hex::ToHex;
use napi_derive::napi;
use std::ops::Deref;
use web3::signing::keccak256;
use web3::types::{Recovery, H256};

#[napi(object)]
pub struct EthersResult {
  pub result: Option<String>,
  pub error: Option<String>,
}

#[napi]
#[allow(dead_code)]
fn private_key_to_public_key(private_key: String) -> EthersResult {
  match internal_private_key_to_public_key(private_key) {
    Ok(val) => EthersResult {
      result: Some(val),
      error: None,
    },
    Err(err) => EthersResult {
      result: None,
      error: Some(format!("{:?}", err)),
    },
  }
}

#[napi]
#[allow(dead_code)]
fn private_key_to_ethers_address(private_key: String) -> EthersResult {
  match internal_private_key_to_ethers_address(private_key) {
    Ok(val) => EthersResult {
      result: Some(val),
      error: None,
    },
    Err(err) => EthersResult {
      result: None,
      error: Some(format!("{:?}", err)),
    },
  }
}

#[napi]
#[allow(dead_code)]
fn signature_to_ethers_signature(sign: String, data: String, chain_id: u32) -> EthersResult {
  match internal_signature_to_ethers_signature(sign, data, chain_id) {
    Ok(val) => EthersResult {
      result: Some(val),
      error: None,
    },
    Err(err) => EthersResult {
      result: None,
      error: Some(format!("{:?}", err)),
    },
  }
}

fn internal_private_key_to_ethers_address(private_key: String) -> anyhow::Result<String> {
  let key: LocalKey<Secp256k1> = serde_json::from_str(&private_key)?;
  let public_key = key.public_key().to_bytes(false);

  debug_assert_eq!(public_key[0], 0x04);
  let hash = keccak256(&public_key[1..]);
  let hash: Vec<u8> = hash.iter().skip(12).cloned().collect();
  let hash: String = hash.encode_hex();

  Ok(String::from("0x") + &hash)
}

fn internal_private_key_to_public_key(private_key: String) -> anyhow::Result<String> {
  let key: LocalKey<Secp256k1> = serde_json::from_str(&private_key)?;

  Ok(key.public_key().to_bytes(true).deref().encode_hex())
}

pub fn sanitize_signature(signature: &mut SignatureRecid, chain: u32) {
  let s = signature.s.to_bigint();
  let n = Secp256k1Scalar::group_order().clone();
  let half_n = n.div_floor(&BigInt::from(2));
  if s > half_n {
    let ns = n - s;
    signature.s = Scalar::<Secp256k1>::from(&ns);
  }

  if signature.recid <= 3 {
    signature.recid += (chain as u8) * 2 + 35;
  }
}

pub fn internal_signature_to_ethers_signature(sign: String, data: String, chain_id: u32) -> anyhow::Result<String> {
  let mut signature: SignatureRecid = serde_json::from_str(&sign)?;
  sanitize_signature(&mut signature, chain_id);

  let rec = Recovery::new(
    data,
    signature.recid.into(),
    H256::from_slice(signature.r.to_bytes().as_ref()),
    H256::from_slice(signature.s.to_bytes().as_ref()),
  );

  let (signature, v) = rec.as_signature().context("failed take signature from recoverable")?;

  let mut slice: [u8; 65] = [0u8; 65];

  slice[..64].copy_from_slice(&signature);
  slice[64] = v as u8;

  Ok(String::from("0x") + &Signature::from_slice(&slice).encode_hex::<String>())
}
