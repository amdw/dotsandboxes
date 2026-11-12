/*
    Copyright 2026 Andrew Medworth <github@medworth.org.uk>

    This file is part of Dots-and-Boxes Engine.

    Dots-and-Boxes Engine is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Dots-and-Boxes Engine is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with Dots-and-Boxes Engine.  If not, see <http://www.gnu.org/licenses/>.
*/
// WebAssembly entry points for the web interface. Each function takes and returns
// plain JavaScript values (converted with serde), and reports errors as exceptions.
pub mod api;

use dabengine::eval::Evaluator;
use dabengine::nimstring::NimstringCalculator;
use dabengine::game::{CPosMove, CompoundPositionData};
use std::cell::RefCell;
use serde::Serialize;
use serde::de::DeserializeOwned;
use wasm_bindgen::prelude::*;

thread_local! {
    // Kept between evaluations, so each can reuse the values cached by earlier ones (this
    // module runs in a single Web Worker)
    static EVALUATOR: RefCell<Evaluator<CPosMove>> = RefCell::new(Evaluator::with_max_cached(api::MAX_CACHED_POSITIONS));
    static NIMSTRING: RefCell<NimstringCalculator> = RefCell::new(NimstringCalculator::with_max_cached(api::MAX_CACHED_POSITIONS));
}

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

fn from_js<T: DeserializeOwned>(value: JsValue) -> Result<T, JsError> {
    serde_wasm_bindgen::from_value(value).map_err(|e| JsError::new(&e.to_string()))
}

fn to_js<T: Serialize>(result: Result<T, String>) -> Result<JsValue, JsError> {
    let value = result.map_err(|e| JsError::new(&e))?;
    // Plain objects rather than Maps, so the results are easy to use from TypeScript
    let serializer = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
    value.serialize(&serializer).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = newGame)]
pub fn new_game(sizes: JsValue) -> Result<JsValue, JsError> {
    let sizes: Vec<(usize, usize)> = from_js(sizes)?;
    to_js(api::new_game(&sizes))
}

#[wasm_bindgen(js_name = legalMoves)]
pub fn legal_moves(position: JsValue) -> Result<JsValue, JsError> {
    let position: CompoundPositionData = from_js(position)?;
    to_js(api::legal_moves(&position))
}

#[wasm_bindgen(js_name = makeMove)]
pub fn make_move(position: JsValue, m: JsValue) -> Result<JsValue, JsError> {
    let position: CompoundPositionData = from_js(position)?;
    let m: CPosMove = from_js(m)?;
    to_js(api::make_move(&position, m))
}

// Evaluate a position, or with max_new, only as far as evaluating that many new positions
// (returning undefined if that isn't enough; see api::evaluate_with)
#[wasm_bindgen]
pub fn evaluate(position: JsValue, max_new: Option<u32>) -> Result<JsValue, JsError> {
    let position: CompoundPositionData = from_js(position)?;
    let max_new = max_new.map(|n| n as usize);
    to_js(EVALUATOR.with(|evaluator| api::evaluate_with(&mut evaluator.borrow_mut(), &position, max_new)))
}

// Calculate Nimstring values (with max_new as for evaluate)
#[wasm_bindgen(js_name = nimstringAnalysis)]
pub fn nimstring_analysis(position: JsValue, max_new: Option<u32>) -> Result<JsValue, JsError> {
    let position: CompoundPositionData = from_js(position)?;
    let max_new = max_new.map(|n| n as usize);
    to_js(NIMSTRING.with(|calculator| api::nimstring_analysis_with(&mut calculator.borrow_mut(), &position, max_new)))
}

// Number of positions evaluated ("evaluate") or whose Nimstring values have been calculated
// ("nimstring") since the module was loaded, e.g. to show the progress of a long calculation
// (as a float, since it may not fit in 32 bits)
#[wasm_bindgen(js_name = positionsCalculated)]
pub fn positions_calculated(kind: &str) -> f64 {
    let count = if kind == "evaluate" {
        EVALUATOR.with(|evaluator| evaluator.borrow().positions_evaluated())
    } else {
        NIMSTRING.with(|calculator| calculator.borrow().positions_calculated())
    };
    count as f64
}

#[wasm_bindgen(js_name = parsePos)]
pub fn parse_pos(text: &str) -> Result<JsValue, JsError> {
    to_js(api::parse_pos(text))
}

#[wasm_bindgen(js_name = canonicalMoves)]
pub fn canonical_moves(position: JsValue) -> Result<JsValue, JsError> {
    let position: CompoundPositionData = from_js(position)?;
    to_js(api::canonical_moves(&position))
}

#[wasm_bindgen]
pub fn split(position: JsValue) -> Result<JsValue, JsError> {
    let position: CompoundPositionData = from_js(position)?;
    to_js(api::split(&position))
}

#[wasm_bindgen(js_name = writePos)]
pub fn write_pos(position: JsValue) -> Result<String, JsError> {
    let position: CompoundPositionData = from_js(position)?;
    api::write_pos(&position).map_err(|e| JsError::new(&e))
}

#[wasm_bindgen]
pub fn examples() -> Result<JsValue, JsError> {
    to_js(Ok(api::examples()))
}
