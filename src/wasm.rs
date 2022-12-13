use crate::keygen::KeygenParams;
use crate::sign::SignParams;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub async fn keygen(params: JsValue) -> Result<JsValue, serde_wasm_bindgen::Error> {
  let params: KeygenParams = serde_wasm_bindgen::from_value(params)?;

  serde_wasm_bindgen::to_value(&crate::keygen::keygen(params).await)
}

#[wasm_bindgen]
pub async fn sign(params: JsValue) -> Result<JsValue, serde_wasm_bindgen::Error> {
  let params: SignParams = serde_wasm_bindgen::from_value(params)?;

  serde_wasm_bindgen::to_value(&crate::sign::sign(params).await)
}

#[allow(dead_code)]
#[wasm_bindgen]
pub fn get_version() -> String {
  env!("CARGO_PKG_VERSION").to_owned()
}

#[wasm_bindgen]
extern "C" {
  // Use `js_namespace` here to bind `console.log(..)` instead of just
  // `log(..)`
  #[wasm_bindgen(js_namespace = console)]
  pub fn log(s: &str);

  // The `console.log` is quite polymorphic, so we can bind it with multiple
  // signatures. Note that we need to use `js_name` to ensure we always call
  // `log` in JS.
  #[wasm_bindgen(js_namespace = console, js_name = log)]
  fn log_u32(a: u32);

  // Multiple arguments too!
  #[wasm_bindgen(js_namespace = console, js_name = log)]
  fn log_many(a: &str, b: &str);
}

// Next let's define a macro that's like `println!`, only it works for
// `console.log`. Note that `println!` doesn't actually work on the wasm target
// because the standard library currently just eats all output. To get
// `println!`-like behavior in your app you'll likely want a macro like this.

#[macro_export]
macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}
