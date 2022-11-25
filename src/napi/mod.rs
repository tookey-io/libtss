use napi_derive::napi;

pub mod ethers;
pub mod keygen;
pub mod sign;

#[napi]
#[allow(dead_code)]
fn get_version() -> String {
  env!("CARGO_PKG_VERSION").to_owned()
}
