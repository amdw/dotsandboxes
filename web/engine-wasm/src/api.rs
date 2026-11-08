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
// The engine operations used by the web interface, on plain data, independent of
// wasm-bindgen so that they can be tested natively. Positions and moves are passed
// in on every call rather than held in the WebAssembly module.
use dabengine::eval;
use dabengine::examples;
use dabengine::game::{CPosMove, CompoundPosition, CompoundPositionData, Position, SimplePosition};
use dabengine::nimstring::{self, ComponentValue, Value};
use dabengine::posfile;
use dabengine::splitter::SplittablePosition;
use serde::Serialize;

// Largest board dimension accepted, to keep the interface (and the engine's
// per-dimension hash seeding) within sensible limits
pub const MAX_DIMENSION: usize = 20;

#[derive(Debug, PartialEq, Serialize)]
pub struct Coin {
    pub part: usize,
    pub x: usize,
    pub y: usize,
}

#[derive(Debug, Serialize)]
pub struct MoveResult {
    // The move as made, in normal form
    #[serde(rename = "move")]
    pub m: CPosMove,
    pub position: CompoundPositionData,
    pub coins_captured: usize,
    // The coins captured by the move
    pub captured: Vec<Coin>,
    pub end_of_turn: bool,
    pub end_of_game: bool,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct MoveValue<V> {
    #[serde(rename = "move")]
    pub m: CPosMove,
    pub value: V,
}

#[derive(Debug, Serialize)]
pub struct Evaluation {
    // Net coins the player to move gains from the rest of the game with optimal play
    pub value: isize,
    pub moves: Vec<MoveValue<isize>>,
}

#[derive(Debug, Serialize)]
pub struct NimstringAnalysis {
    pub value: Value,
    pub components: Vec<ComponentValue>,
    pub moves: Vec<MoveValue<Value>>,
}

#[derive(Debug, Serialize)]
pub struct Example {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub position: CompoundPositionData,
    pub moves: Vec<CPosMove>,
}

fn check_dimensions(width: usize, height: usize) -> Result<(), String> {
    if width == 0 || height == 0 || width > MAX_DIMENSION || height > MAX_DIMENSION {
        Err(format!("Dimensions must be between 1 and {}, got {}x{}", MAX_DIMENSION, width, height))
    } else {
        Ok(())
    }
}

fn position_from(data: &CompoundPositionData) -> Result<CompoundPosition, String> {
    for part in &data.parts {
        check_dimensions(part.width, part.height)?;
    }
    CompoundPosition::from_data(data)
}

// A new game, with one part of each of the given sizes
pub fn new_game(sizes: &[(usize, usize)]) -> Result<CompoundPositionData, String> {
    if sizes.is_empty() {
        return Err("A position must have at least one part".to_string());
    }
    let mut parts = Vec::with_capacity(sizes.len());
    for &(width, height) in sizes {
        check_dimensions(width, height)?;
        parts.push(SimplePosition::new_game(width, height));
    }
    Ok(CompoundPosition::new(parts).to_data())
}

pub fn legal_moves(data: &CompoundPositionData) -> Result<Vec<CPosMove>, String> {
    Ok(position_from(data)?.legal_moves())
}

pub fn make_move(data: &CompoundPositionData, m: CPosMove) -> Result<MoveResult, String> {
    let mut pos = position_from(data)?;
    let m = pos.normalise_move(m).ok_or(format!("Move off the board: {}", m))?;
    if !pos.is_legal_move(m) {
        return Err(format!("Not a legal move: {}", m));
    }
    let part = &pos.parts()[m.part];
    let mut candidates = vec![(m.m.x, m.m.y)];
    if let Some(neighbour) = part.offset(m.m.x, m.m.y, m.m.side) {
        candidates.push(neighbour);
    }
    let outcome = pos.make_move(m);
    let part = &pos.parts()[m.part];
    let captured = candidates.into_iter()
        .filter(|&(x, y)| part.is_captured(x, y))
        .map(|(x, y)| Coin{ part: m.part, x: x, y: y })
        .collect();
    Ok(MoveResult{
        m: m,
        position: pos.to_data(),
        coins_captured: outcome.coins_captured,
        captured: captured,
        end_of_turn: outcome.end_of_turn,
        end_of_game: outcome.end_of_game,
    })
}

pub fn evaluate(data: &CompoundPositionData) -> Result<Evaluation, String> {
    let pos = position_from(data)?;
    let (value, per_move) = eval::eval_with_moves(&pos);
    let moves = pos.legal_moves().into_iter().map(|m| MoveValue{ m: m, value: per_move[&m] }).collect();
    Ok(Evaluation{ value: value, moves: moves })
}

pub fn nimstring_analysis(data: &CompoundPositionData) -> Result<NimstringAnalysis, String> {
    let pos = position_from(data)?;
    let (value, per_move) = nimstring::calc_value_with_moves(&pos);
    let moves = pos.legal_moves().into_iter().map(|m| MoveValue{ m: m, value: per_move[&m] }).collect();
    Ok(NimstringAnalysis{ value: value, components: nimstring::component_values(&pos), moves: moves })
}

pub fn parse_pos(text: &str) -> Result<CompoundPositionData, String> {
    let pos = posfile::parse_str(text)?;
    for part in pos.parts() {
        check_dimensions(part.width(), part.height())?;
    }
    Ok(pos.to_data())
}

pub fn write_pos(data: &CompoundPositionData) -> Result<String, String> {
    Ok(posfile::write(&position_from(data)?))
}

// The position split into its independent components, one part each (cropped to the
// component's bounding box), or None if that wouldn't give more parts than it has already
pub fn split(data: &CompoundPositionData) -> Result<Option<CompoundPositionData>, String> {
    let pos = position_from(data)?;
    let parts = pos.split();
    if parts.len() <= pos.parts().len() {
        return Ok(None);
    }
    Ok(Some(CompoundPosition::new(parts).to_data()))
}

pub fn examples() -> Vec<Example> {
    examples::catalogue().into_iter().map(|e| Example{
        id: e.id,
        title: e.title,
        description: e.description,
        position: e.position.to_data(),
        moves: e.moves,
    }).collect()
}

#[cfg(test)]
mod tests {
    use crate::api::*;
    use dabengine::game::Side;

    fn chain(length: usize) -> CompoundPositionData {
        CompoundPosition::from_single(examples::make_chain(length)).to_data()
    }

    #[test]
    fn new_game_sizes() {
        let pos = new_game(&[(3, 2)]).unwrap();
        assert_eq!(1, pos.parts.len());
        assert_eq!(17, legal_moves(&pos).unwrap().len());
        assert_eq!(2, new_game(&[(1, 1), (2, 2)]).unwrap().parts.len());
        assert!(new_game(&[]).is_err());
        assert!(new_game(&[(0, 2)]).is_err());
        assert!(new_game(&[(2, MAX_DIMENSION + 1)]).is_err());
    }

    #[test]
    fn make_move_normalises_and_reports_captures() {
        let pos = chain(2);
        // Open the chain at the right-hand end
        let result = make_move(&pos, CPosMove::new(0, 1, 0, Side::Right)).unwrap();
        assert_eq!(0, result.coins_captured);
        assert!(result.end_of_turn);
        assert!(result.captured.is_empty());
        // Cut the middle string, described from the right-hand coin: captures that coin
        let result = make_move(&result.position, CPosMove::new(0, 1, 0, Side::Left)).unwrap();
        assert_eq!(CPosMove::new(0, 0, 0, Side::Right), result.m);
        assert_eq!(1, result.coins_captured);
        assert_eq!(vec![Coin{ part: 0, x: 1, y: 0 }], result.captured);
        assert!(!result.end_of_turn);
        assert!(!result.end_of_game);
        // Take the last coin
        let result = make_move(&result.position, CPosMove::new(0, 0, 0, Side::Left)).unwrap();
        assert_eq!(vec![Coin{ part: 0, x: 0, y: 0 }], result.captured);
        assert!(result.end_of_game);
    }

    #[test]
    fn make_move_double_capture() {
        let mut pos = SimplePosition::new_game(2, 1);
        for m in pos.legal_moves() {
            if m != dabengine::game::Move::new(0, 0, Side::Right) {
                pos.make_move(m);
            }
        }
        let data = CompoundPosition::from_single(pos).to_data();
        let result = make_move(&data, CPosMove::new(0, 0, 0, Side::Right)).unwrap();
        assert_eq!(2, result.coins_captured);
        assert_eq!(vec![Coin{ part: 0, x: 0, y: 0 }, Coin{ part: 0, x: 1, y: 0 }], result.captured);
        assert!(result.end_of_game);
    }

    #[test]
    fn make_move_errors() {
        let pos = chain(2);
        assert!(make_move(&pos, CPosMove::new(0, 0, 0, Side::Top)).err().unwrap().contains("Not a legal move"));
        assert!(make_move(&pos, CPosMove::new(1, 0, 0, Side::Left)).err().unwrap().contains("off the board"));
        assert!(make_move(&pos, CPosMove::new(0, 5, 0, Side::Left)).err().unwrap().contains("off the board"));
    }

    #[test]
    fn evaluation() {
        // Closed 3-chain: worth -3 to the player to move, whatever they do
        let result = evaluate(&chain(3)).unwrap();
        assert_eq!(-3, result.value);
        assert_eq!(4, result.moves.len());
        assert!(result.moves.iter().all(|mv| mv.value == -3));
    }

    #[test]
    fn nimstring() {
        let result = nimstring_analysis(&CompoundPosition::from_single(examples::p50()).to_data()).unwrap();
        assert_eq!(Value::Nimber(7), result.value);
        assert_eq!(3, result.components.len());
        assert_eq!(legal_moves(&CompoundPosition::from_single(examples::p50()).to_data()).unwrap().len(),
                   result.moves.len());
    }

    #[test]
    fn pos_text_round_trip() {
        let pos = chain(3);
        let text = write_pos(&pos).unwrap();
        assert_eq!(pos, parse_pos(&text).unwrap());
        assert!(parse_pos("21 1").is_err());
    }

    #[test]
    fn split_components() {
        // The page 50 example has three independent components
        let p50 = CompoundPosition::from_single(examples::p50());
        let parts = split(&p50.to_data()).unwrap().unwrap();
        assert_eq!(3, parts.parts.len());
        // Splitting doesn't change the Nimstring value
        assert_eq!(nimstring_analysis(&p50.to_data()).unwrap().value, nimstring_analysis(&parts).unwrap().value);
        // Already split, a single component, or nothing left: no split
        assert!(split(&parts).unwrap().is_none());
        assert!(split(&chain(3)).unwrap().is_none());
        assert!(split(&CompoundPosition::from_single(SimplePosition::new_end_game(2, 2)).to_data()).unwrap().is_none());
        // Two chains in one part split into two
        let double = CompoundPosition::from_single(examples::double_chain(3)).to_data();
        assert_eq!(Some(2), split(&double).unwrap().map(|s| s.parts.len()));
    }

    #[test]
    fn example_list() {
        let list = examples();
        assert!(list.iter().any(|e| e.id == "berlekamp-p50"));
        assert!(list.iter().any(|e| !e.moves.is_empty()));
    }
}
