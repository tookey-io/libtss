pub mod ceremonies;
pub mod ecdsa;
pub mod join;
pub mod keygen;
pub mod sign;

use napi_derive::napi;

#[napi]
fn get_version() -> String {
    option_env!("CARGO_PKG_VERSION").unwrap().to_owned()
}
