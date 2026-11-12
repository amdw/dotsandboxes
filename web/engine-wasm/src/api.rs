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
use dabengine::eval::{self, Evaluator};
use dabengine::examples;
use dabengine::game::{CPosMove, CompoundPosition, CompoundPositionData, Position, SimplePosition};
use dabengine::nimstring::{ComponentValue, NimstringCalculator, Value};
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

// Most positions cached by the evaluator and the Nimstring calculator kept between
// calculations (see Evaluator::with_max_cached), to bound their memory use. Beyond this, they
// forget the values cheapest to calculate again. This fills a hash table of 2^24 entries,
// about 550MB for the evaluator in WebAssembly, the next size up being too big to allocate
// there (and too big for many phones).
pub const MAX_CACHED_POSITIONS: usize = (1 << 24) / 8 * 7;

// Evaluate a position with an evaluator kept between evaluations, so later evaluations
// (e.g. of the positions reached as a game is played on) can reuse its cached values.
// With max_new, give up (returning None) if that would need more than that many positions
// which aren't already cached: i.e. only evaluate it if that's cheap, or evaluate a long
// calculation in chunks, each carrying on from what earlier ones cached, until done.
pub fn evaluate_with(evaluator: &mut Evaluator<CPosMove>, data: &CompoundPositionData, max_new: Option<usize>) -> Result<Option<Evaluation>, String> {
    let pos = position_from(data)?;
    let Some((value, per_move)) = evaluator.eval_with_moves_limited(&pos, max_new.unwrap_or(usize::MAX)) else {
        return Ok(None);
    };
    let moves = pos.legal_moves().into_iter().map(|m| MoveValue{ m: m, value: per_move[&m] }).collect();
    Ok(Some(Evaluation{ value: value, moves: moves }))
}

pub fn evaluate(data: &CompoundPositionData) -> Result<Evaluation, String> {
    Ok(evaluate_with(&mut Evaluator::new(), data, None)?.unwrap())
}

// Calculate Nimstring values with a calculator kept between calculations (see evaluate_with)
pub fn nimstring_analysis_with(calculator: &mut NimstringCalculator, data: &CompoundPositionData, max_new: Option<usize>) -> Result<Option<NimstringAnalysis>, String> {
    let pos = position_from(data)?;
    let Some(analysis) = calculator.analysis_limited(&pos, max_new.unwrap_or(usize::MAX)) else {
        return Ok(None);
    };
    let moves = pos.legal_moves().into_iter().map(|m| MoveValue{ m: m, value: analysis.per_move[&m] }).collect();
    Ok(Some(NimstringAnalysis{ value: analysis.value, components: analysis.components, moves: moves }))
}

pub fn nimstring_analysis(data: &CompoundPositionData) -> Result<NimstringAnalysis, String> {
    Ok(nimstring_analysis_with(&mut NimstringCalculator::new(), data, None)?.unwrap())
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

// The canonical captures in a position: those which can be made without loss
pub fn canonical_moves(data: &CompoundPositionData) -> Result<Vec<CPosMove>, String> {
    Ok(eval::canonical_moves(&position_from(data)?))
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
    fn evaluation_with_kept_evaluator() {
        // Playing on from a position, a kept evaluator gives the same values as a fresh one
        let mut evaluator = Evaluator::new();
        let mut pos = CompoundPosition::from_single(examples::ex3p12()).to_data();
        for _ in 0..5 {
            let kept = evaluate_with(&mut evaluator, &pos, None).unwrap().unwrap();
            let fresh = evaluate(&pos).unwrap();
            assert_eq!(fresh.value, kept.value);
            assert_eq!(fresh.moves, kept.moves);
            let best = kept.moves.iter().find(|m| m.value == kept.value).unwrap().m;
            pos = make_move(&pos, best).unwrap().position;
        }
        assert!(evaluator.cached_positions() > 0);
    }

    #[test]
    fn limited_calculations() {
        let pos = CompoundPosition::from_single(examples::ex3p12()).to_data();
        // Too expensive with a small budget...
        assert!(evaluate_with(&mut Evaluator::new(), &pos, Some(10)).unwrap().is_none());
        assert!(nimstring_analysis_with(&mut NimstringCalculator::new(), &pos, Some(10)).unwrap().is_none());
        // ...but cheap once cached
        let mut evaluator = Evaluator::new();
        let full = evaluate_with(&mut evaluator, &pos, None).unwrap().unwrap();
        assert_eq!(full.moves, evaluate_with(&mut evaluator, &pos, Some(0)).unwrap().unwrap().moves);
        let mut calculator = NimstringCalculator::new();
        let full = nimstring_analysis_with(&mut calculator, &pos, None).unwrap().unwrap();
        let again = nimstring_analysis_with(&mut calculator, &pos, Some(0)).unwrap().unwrap();
        assert_eq!((full.value, full.moves, full.components), (again.value, again.moves, again.components));
    }

    #[test]
    fn chunked_calculations() {
        // A long calculation done in small chunks, continuing until it completes, gives the
        // same result as doing it in one go
        let pos = CompoundPosition::from_single(examples::ex3p12()).to_data();
        let full = evaluate(&pos).unwrap();
        let mut evaluator = Evaluator::new();
        let mut chunks = 0;
        let chunked = loop {
            chunks += 1;
            if let Some(result) = evaluate_with(&mut evaluator, &pos, Some(200)).unwrap() {
                break result;
            }
        };
        assert!(chunks > 1);
        assert_eq!((full.value, full.moves), (chunked.value, chunked.moves));

        let full = nimstring_analysis(&pos).unwrap();
        let mut calculator = NimstringCalculator::new();
        let mut chunks = 0;
        let chunked = loop {
            chunks += 1;
            if let Some(result) = nimstring_analysis_with(&mut calculator, &pos, Some(50)).unwrap() {
                break result;
            }
        };
        assert!(chunks > 1);
        assert_eq!((full.value, full.moves, full.components), (chunked.value, chunked.moves, chunked.components));

        // Likewise with too small a cache to keep everything, so values are forgotten
        let full = evaluate(&pos).unwrap();
        let mut evaluator = Evaluator::with_max_cached(2000);
        let chunked = loop {
            if let Some(result) = evaluate_with(&mut evaluator, &pos, Some(200)).unwrap() {
                break result;
            }
        };
        assert!(evaluator.positions_evaluated() > 2000);
        assert_eq!((full.value, full.moves), (chunked.value, chunked.moves));
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
    fn canonical() {
        let opened = make_move(&chain(3), CPosMove::new(0, 0, 0, Side::Left)).unwrap().position;
        assert_eq!(vec![CPosMove::new(0, 0, 0, Side::Right)], canonical_moves(&opened).unwrap());
        assert!(canonical_moves(&chain(3)).unwrap().is_empty());
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
