pub mod ceremonies;
pub mod ecdsa;
pub mod join;
pub mod keygen;
pub mod sign;

use napi_derive::napi;

#[napi]
fn get_version() -> String {
  env!("CARGO_PKG_VERSION").to_owned()
}
