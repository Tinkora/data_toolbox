use serde::Serialize;
use serde::de::DeserializeOwned;
use wasm_bindgen::prelude::*;

use data_toolbox_core::{ConvertOptions, CoreError, InspectOptions, convert, inspect};

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn inspect_csv(input: &str, options_json: &str) -> JsValue {
    let response =
        decode_options::<InspectOptions>(options_json).and_then(|options| inspect(input, &options));
    respond(response)
}

#[wasm_bindgen]
pub fn convert_csv(input: &str, options_json: &str) -> JsValue {
    let response =
        decode_options::<ConvertOptions>(options_json).and_then(|options| convert(input, &options));
    respond(response)
}

fn decode_options<T: DeserializeOwned>(options_json: &str) -> Result<T, CoreError> {
    serde_json::from_str(options_json).map_err(|_| CoreError::InvalidOptions)
}

#[derive(Serialize)]
struct Success<'a, T> {
    ok: bool,
    data: &'a T,
}

#[derive(Serialize)]
struct Failure {
    ok: bool,
    error: data_toolbox_core::ErrorEnvelope,
}

fn respond<T: Serialize>(result: Result<T, CoreError>) -> JsValue {
    let value = match result {
        Ok(data) => serde_wasm_bindgen::to_value(&Success {
            ok: true,
            data: &data,
        }),
        Err(error) => serde_wasm_bindgen::to_value(&Failure {
            ok: false,
            error: error.to_envelope(),
        }),
    };
    value.unwrap_or(JsValue::NULL)
}
