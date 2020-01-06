extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn shift(x: i32) -> i32 {
    ((x << 1) << 1)
}
