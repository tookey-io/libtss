pub mod ceremonies;
pub mod ecdsa;
pub mod ethers;
pub mod join;
pub mod keygen;
pub mod sign;

pub use curv;
pub use ethereum_types;
pub use round_based;

use napi_derive::napi;

#[napi]
#[allow(dead_code)]
fn get_version() -> String {
  env!("CARGO_PKG_VERSION").to_owned()
}
