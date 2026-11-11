/*
    Copyright 2017-2020, 2026 Andrew Medworth <github@medworth.org.uk>

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
use crate::game::{Position, SimplePosition, CompoundPosition, Side, Move, CPosMove};
use crate::splitter::{self, SplittablePosition};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::Hash;
use std::ops;

#[derive(Clone)]
#[derive(Copy)]
#[derive(PartialEq)]
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Value {
    Nimber(usize),
    Loony
}

impl ops::Add for Value {
    type Output = Value;

    fn add(self: Value, other: Value) -> Value {
        match (self, other) {
            (Value::Loony, _) => Value::Loony,
            (_, Value::Loony) => Value::Loony,
            (Value::Nimber(x), Value::Nimber(y)) => Value::Nimber(x ^ y),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(self: &Value, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Value::Nimber(v) => write!(f, "*{}", v),
            &Value::Loony => write!(f, "L"),
        }
    }
}

pub trait NimstringPosition<M>: SplittablePosition<M> {
    // Indicate whether a given position is loony
    fn is_loony(&self) -> bool;
}

impl NimstringPosition<Move> for SimplePosition {
    fn is_loony(self: &SimplePosition) -> bool {
        for x in 0..self.width() {
            for y in 0..self.height() {
                if self.valency(x, y) != 1 {
                    continue
                }
                if let Some((neighbour_x, neighbour_y, side)) = connected_coin(self, x, y, Side::ALL) {
                    if self.valency(neighbour_x, neighbour_y) != 2 {
                        continue
                    }
                    // We have found a capturable coin attached to a coin of valency 2 (o-o-?).
                    // This means the position is loony unless there is a valency-1 coin
                    // on the other side (o-o-o).
                    let far_sides = Side::all_except(side.opposite());
                    if let Some((far_x, far_y, _)) = connected_coin(self, neighbour_x, neighbour_y, far_sides) {
                        if self.valency(far_x, far_y) == 1 {
                            continue;
                        }
                    }
                    return true;
                }
            }
        }
        false
    }
}

impl NimstringPosition<CPosMove> for CompoundPosition {
    fn is_loony(self: &CompoundPosition) -> bool {
        // A CompoundPosition is loony iff any of its parts is loony
        for part in self.parts.iter() {
            if part.is_loony() {
                return true;
            }
        }
        false
    }
}

// If there is a coin connected to (x,y) on one of the given sides, return one such, else None.
fn connected_coin(pos: &SimplePosition, x: usize, y: usize, sides: impl IntoIterator<Item = Side>) -> Option<(usize, usize, Side)> {
    for s in sides {
        if pos.is_legal_move(Move{x: x, y: y, side: s}) {
            if let Some((nx, ny)) = pos.offset(x, y, s) {
                return Some((nx, ny, s));
            }
        }
    }
    None
}

// Indicate whether a given move would result in a loony position
// (regardless of whether the current position is loony).
// Note that a move is loony iff it returns true here *and* it is not a capture.
// A capture is never a loony move but can return true here (e.g. capturing
// the first coin of an open 3-chain).
pub fn would_be_loony<M, P>(pos: &mut P, m: M) -> bool
where M: Copy, P: NimstringPosition<M> {
    pos.make_move(m);
    let result = pos.is_loony();
    pos.undo_move(m);
    result
}

// Minimal excludant helper function
fn mex(s: HashSet<usize>) -> usize {
    let mut i = 0;
    loop {
        if !s.contains(&i) {
            return i;
        }
        i += 1;
    }
}

// The state of a Nimstring calculation: the cache of position values, and a limit on its
// size, beyond which the calculation is abandoned (aborted). Once aborted, the search unwinds
// without caching anything more, so every cached value is still exact.
struct NimSearch<'a> {
    cache: &'a mut HashMap<u64, Value>,
    limit: usize,
    aborted: bool,
}

fn calc_value<M, P>(pos: &mut P, search: &mut NimSearch) -> Value
where M: Copy, P: NimstringPosition<M> {
    // TODO: Optimise by iterating over a tighter set of moves than all legal moves
    if let Some(&v) = search.cache.get(&pos.zhash()) {
        return v;
    }
    if search.cache.len() >= search.limit {
        search.aborted = true;
        return Value::Nimber(0);
    }
    if pos.is_loony() {
        search.cache.insert(pos.zhash(), Value::Loony);
        return Value::Loony;
    }

    let legal_moves = pos.legal_moves();
    for &m in &legal_moves {
        if pos.would_capture(m) > 0 {
            pos.make_move(m);
            let result = calc_value(pos, search);
            pos.undo_move(m);
            if !search.aborted {
                search.cache.insert(pos.zhash(), result);
            }
            return result
        }
    }

    // Try to split the position into independent parts which can be evaluated separately
    let parts = pos.split();
    if parts.len() > 1 {
        let mut result = Value::Nimber(0);
        for mut part in parts {
            let part_value = calc_value(&mut part, search);
            if search.aborted {
                return result;
            }
            result = result + part_value;
        }
        search.cache.insert(pos.zhash(), result);
        return result;
    }

    // Moves which give the same position as another have the same value, so needn't be tried
    let mut options = HashSet::new();
    for m in pos.distinct_moves() {
        pos.make_move(m);
        let value = calc_value(pos, search);
        pos.undo_move(m);
        if search.aborted {
            return value;
        }
        if let Value::Nimber(n) = value {
            options.insert(n);
        }
    }
    let result = Value::Nimber(mex(options));
    search.cache.insert(pos.zhash(), result);
    result
}

// The Nimstring value of one independent component of a position
#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ComponentValue {
    // Which part of the CompoundPosition the component is in
    pub part: usize,
    // Coordinates within that part of the coins making up the component
    pub coins: Vec<(usize, usize)>,
    pub value: Value,
}

// The Nimstring values of a position, of each legal move from it, and of each of its
// independent components
#[derive(Debug)]
#[derive(PartialEq)]
pub struct NimstringAnalysis<M: Hash + Eq> {
    pub value: Value,
    pub per_move: HashMap<M, Value>,
    pub components: Vec<ComponentValue>,
}

// A Nimstring calculator which keeps the values of the positions it has calculated, so that
// later calculations (e.g. of the positions reached as a game goes on) can reuse them. The
// value of a position depends only on the position, so the cache never goes stale.
pub struct NimstringCalculator {
    cache: HashMap<u64, Value>,
}

impl NimstringCalculator {
    pub fn new() -> NimstringCalculator {
        NimstringCalculator{ cache: HashMap::new() }
    }

    // Number of positions whose values are cached
    pub fn cached_positions(&self) -> usize {
        self.cache.len()
    }

    // Forget all cached values, freeing their memory
    pub fn clear(&mut self) {
        self.cache = HashMap::new();
    }

    fn search(&mut self, max_new: usize) -> NimSearch<'_> {
        let limit = self.cache.len().saturating_add(max_new);
        NimSearch{ cache: &mut self.cache, limit: limit, aborted: false }
    }

    // Calculate the Nimstring value of a position, along with the values attained
    // by each of the legal moves.
    pub fn value_with_moves<M, P>(&mut self, pos: &P) -> (Value, HashMap<M, Value>)
    where M: Hash + Eq + Copy, P: NimstringPosition<M> + Clone {
        let mut search = self.search(usize::MAX);
        value_with_moves(pos, &mut search)
    }

    // Calculate the Nimstring values of the independent components of a position.
    // The value of the whole position is the sum of these values.
    pub fn component_values(&mut self, pos: &CompoundPosition) -> Vec<ComponentValue> {
        let mut search = self.search(usize::MAX);
        component_values_in(pos, &mut search)
    }

    // The value of a position, each move and each component, giving up (returning None) if
    // that would need more than max_new positions which aren't already cached: i.e.
    // calculate it only if that's cheap. The values calculated before giving up stay cached.
    pub fn analysis_limited(&mut self, pos: &CompoundPosition, max_new: usize) -> Option<NimstringAnalysis<CPosMove>> {
        let mut search = self.search(max_new);
        let (value, per_move) = value_with_moves(pos, &mut search);
        if search.aborted {
            return None;
        }
        let components = component_values_in(pos, &mut search);
        if search.aborted { None } else { Some(NimstringAnalysis{ value: value, per_move: per_move, components: components }) }
    }
}

fn value_with_moves<M, P>(pos: &P, search: &mut NimSearch) -> (Value, HashMap<M, Value>)
where M: Hash + Eq + Copy, P: NimstringPosition<M> + Clone {
    let mut pos = pos.clone();
    let val = calc_value(&mut pos, search);
    let mut per_move = HashMap::new();
    for m in pos.legal_moves() {
        if search.aborted {
            break;
        }
        pos.make_move(m);
        per_move.insert(m, calc_value(&mut pos, search));
        pos.undo_move(m);
    }
    (val, per_move)
}

fn component_values_in(pos: &CompoundPosition, search: &mut NimSearch) -> Vec<ComponentValue> {
    let mut result = Vec::new();
    for (i, part) in pos.parts().iter().enumerate() {
        for mut fragment in splitter::fragments(part) {
            if search.aborted {
                return result;
            }
            let value = calc_value(&mut fragment.position, search);
            result.push(ComponentValue{ part: i, coins: fragment.coins, value: value });
        }
    }
    result
}

// Calculate the Nimstring value of a position, along with the values attained
// by each of the legal moves.
pub fn calc_value_with_moves<M, P>(pos: &P) -> (Value, HashMap<M, Value>)
where M: Hash + Eq + Copy, P: NimstringPosition<M> + Clone {
    NimstringCalculator::new().value_with_moves(pos)
}

// Calculate the Nimstring values of the independent components of a position.
// The value of the whole position is the sum of these values.
pub fn component_values(pos: &CompoundPosition) -> Vec<ComponentValue> {
    NimstringCalculator::new().component_values(pos)
}

#[cfg(test)]
mod tests {
    use crate::nimstring::*;
    use crate::game::*;
    use crate::examples::*;

    #[test]
    fn value_display() {
        assert_eq!("*0", format!("{}", Value::Nimber(0)));
        assert_eq!("*1", format!("{}", Value::Nimber(1)));
        assert_eq!("*2", format!("{}", Value::Nimber(2)));
        assert_eq!("L", format!("{}", Value::Loony));
    }

    #[test]
    fn min_excl() {
        assert_eq!(0, mex(HashSet::new()));
        assert_eq!(0, mex([1,2,3].iter().cloned().collect()));
        assert_eq!(1, mex([0].iter().cloned().collect()));
        assert_eq!(1, mex([0,2,3].iter().cloned().collect()));
        assert_eq!(2, mex([0,1,3].iter().cloned().collect()));
        assert_eq!(3, mex([0,1,2].iter().cloned().collect()));
    }

    #[test]
    fn nimber_addition() {
        assert_eq!(Value::Nimber(3), Value::Nimber(1) + Value::Nimber(2));
        assert_eq!(Value::Nimber(2), Value::Nimber(1) + Value::Nimber(3));
        assert_eq!(Value::Nimber(1), Value::Nimber(2) + Value::Nimber(3));

        assert_eq!(Value::Nimber(6), Value::Nimber(2) + Value::Nimber(4));
        assert_eq!(Value::Nimber(4), Value::Nimber(6) + Value::Nimber(2));
        assert_eq!(Value::Nimber(2), Value::Nimber(4) + Value::Nimber(6));

        assert_eq!(Value::Loony, Value::Nimber(2) + Value::Loony);
        assert_eq!(Value::Loony, Value::Loony + Value::Nimber(2));
    }

    #[test]
    fn basic_values() {
        let mut pos = make_chain(3);
        assert!(!pos.is_loony());
        let mut cache = HashMap::new();
        let mut search = NimSearch{ cache: &mut cache, limit: usize::MAX, aborted: false };
        assert_eq!(Value::Nimber(0), calc_value(&mut pos, &mut search));
        pos.make_move(Move{x: 0, y: 0, side: Side::Left});
        assert!(pos.is_loony());
        assert_eq!(Value::Loony, calc_value(&mut pos, &mut search));
        pos.make_move(Move{x: 1, y: 0, side: Side::Left});
        assert!(pos.is_loony());
        assert_eq!(Value::Loony, calc_value(&mut pos, &mut search));
        pos.make_move(Move{x: 2, y: 0, side: Side::Left});
        assert!(!pos.is_loony());
        assert_eq!(Value::Nimber(0), calc_value(&mut pos, &mut search));
        pos.make_move(Move{x: 2, y: 0, side: Side::Right});
        assert!(!pos.is_loony());
        assert_eq!(Value::Nimber(0), calc_value(&mut pos, &mut search));
    }

    #[test]
    fn open_3loop_not_loony() {
        let mut pos = make_chain(3);
        pos.make_move(Move{x: 0, y: 0, side: Side::Left});
        pos.make_move(Move{x: 2, y: 0, side: Side::Right});
        assert_eq!(false, pos.is_loony());
        let (val, _) = calc_value_with_moves(&pos);
        assert_eq!(Value::Nimber(0), val);
    }

    #[test]
    fn nonzero_value() {
        let mut pos = make_chain(7);
        pos.undo_move(Move{x: 3, y: 0, side: Side::Top});
        let (val, per_move) = calc_value_with_moves(&pos);
        assert_eq!(Value::Nimber(1), val);
        assert_eq!(&Value::Nimber(0), per_move.get(&Move{x: 3, y: 0, side: Side::Top}).unwrap());
    }

    #[test]
    fn right_capture_detection() {
        let mut pos = make_chain(5);
        pos.undo_move(Move{x: 3, y: 0, side: Side::Top});
        let (val, per_move) = calc_value_with_moves(&pos);
        assert_eq!(Value::Nimber(1), val);
        assert_eq!(&Value::Nimber(0), per_move.get(&Move{x: 4, y: 0, side: Side::Right}).unwrap());
    }

    #[test]
    fn p50_top_value() {
        let pos = p50_top();
        let (val, per_move) = calc_value_with_moves(&pos);
        assert_eq!(Value::Nimber(1), val);
        assert_eq!(&Value::Nimber(0), per_move.get(&Move{x: 3, y: 0, side: Side::Top}).unwrap());
    }

    #[test]
    fn p50_bottomleft_value() {
        let pos = p50_bottomleft();
        let (val, per_move) = calc_value_with_moves(&pos);
        assert_eq!(Value::Nimber(4), val);
        assert_eq!(&Value::Nimber(3), per_move.get(&Move{x: 0, y: 1, side: Side::Left}).unwrap());
        assert_eq!(&Value::Nimber(3), per_move.get(&Move{x: 0, y: 1, side: Side::Bottom}).unwrap());
        assert_eq!(&Value::Nimber(3),
                   per_move.get(&Move{x: 0, y: 1, side: Side::Right}).or(per_move.get(&Move{x: 0, y: 2, side: Side::Left})).unwrap());
    }

    #[test]
    fn p50_bottomright_value() {
        let pos = p50_bottomright();
        let (val, per_move) = calc_value_with_moves(&pos);
        assert_eq!(Value::Nimber(2), val);
        assert_eq!(&Value::Nimber(3), per_move.get(&Move{x: 0, y: 1, side: Side::Right}).unwrap());
    }

    #[test]
    fn p50_value() {
        let pos = p50();
        let (val, per_move) = calc_value_with_moves(&pos);
        assert_eq!(Value::Nimber(7), val);
        let zero_moves: Vec<&Move> = per_move.keys().filter(|m| &Value::Nimber(0) == per_move.get(m).unwrap()).collect();
        assert_eq!(3, zero_moves.len());
        assert!(zero_moves.contains(&&Move{x: 0, y: 3, side: Side::Bottom}));
        assert!(zero_moves.contains(&&Move{x: 0, y: 3, side: Side::Left}));
        assert!(zero_moves.contains(&&Move{x: 0, y: 3, side: Side::Right}));
    }

    #[test]
    fn icelandic_value_2by2() {
       let pos = icelandic_game(2, 2);
       let (val, _per_move) = calc_value_with_moves(&pos);
       assert_eq!(Value::Nimber(2), val);
    }

    #[test]
    fn ex6p1_value() {
       let mut pos = ex6p2();
       pos.make_move(Move{x: 1, y: 1, side: Side::Left}); // Same position but rotated
       let (val, _per_move) = calc_value_with_moves(&pos);
       assert_eq!(Value::Nimber(3), val);
    }

    #[test]
    fn ex6p2_value() {
       let pos = ex6p2();
       let (val, _per_move) = calc_value_with_moves(&pos);
       assert_eq!(Value::Nimber(4), val);
    }

    #[test]
    fn ex7p2_value() {
       let pos = ex7p2();
       let (val, per_move) = calc_value_with_moves(&pos);
       assert_eq!(Value::Nimber(6), val);
       assert_eq!(&Value::Nimber(0), per_move.get(&Move{x: 4, y: 3, side: Side::Right}).unwrap());
    }

    #[test]
    fn conditional_looniness() {
        let mut pos = make_chain(5);
        assert_eq!(true, would_be_loony(&mut pos, Move{x: 0, y: 0, side: Side::Left}));
        pos.make_move(Move{x: 0, y: 0, side: Side::Left});
        assert_eq!(true, pos.is_loony());
        assert_eq!(true, would_be_loony(&mut pos, Move{x: 1, y: 0, side: Side::Left}));
        pos.make_move(Move{x: 1, y: 0, side: Side::Left});
        assert_eq!(true, pos.is_loony());
        assert_eq!(true, would_be_loony(&mut pos, Move{x: 2, y: 0, side: Side::Left}));
        pos.make_move(Move{x: 2, y: 0, side: Side::Left});
        assert_eq!(true, pos.is_loony());
        assert_eq!(true, would_be_loony(&mut pos, Move{x: 3, y: 0, side: Side::Left}));
        pos.make_move(Move{x: 3, y: 0, side: Side::Left});
        assert_eq!(true, pos.is_loony());
        assert_eq!(false, would_be_loony(&mut pos, Move{x: 4, y: 0, side: Side::Left}));
        assert_eq!(false, would_be_loony(&mut pos, Move{x: 4, y: 0, side: Side::Right}));
        pos.make_move(Move{x: 4, y: 0, side: Side::Left});
        assert_eq!(false, pos.is_loony());
        assert_eq!(false, would_be_loony(&mut pos, Move{x: 4, y: 0, side: Side::Right}));
        pos.make_move(Move{x: 4, y: 0, side: Side::Right});
        assert_eq!(true, pos.is_end_of_game());
        assert_eq!(false, pos.is_loony());
    }

    #[test]
    fn p50_component_values() {
        let pos = CompoundPosition::from_single(p50());
        let components = component_values(&pos);
        let mut values: Vec<Value> = components.iter().map(|c| c.value).collect();
        values.sort_by_key(|v| match v { Value::Nimber(n) => *n, Value::Loony => usize::MAX });
        assert_eq!(vec![Value::Nimber(1), Value::Nimber(2), Value::Nimber(4)], values);
        assert!(components.iter().all(|c| c.part == 0));
        let total = components.iter().fold(Value::Nimber(0), |acc, c| acc + c.value);
        assert_eq!(calc_value_with_moves(&pos).0, total);
    }

    #[test]
    fn compound_component_values() {
        let mut pos = CompoundPosition::new(vec!(make_chain(2), double_chain(3)));
        let components = component_values(&pos);
        assert_eq!(3, components.len());
        assert_eq!(ComponentValue{ part: 0, coins: vec![(0, 0), (1, 0)], value: Value::Nimber(1) }, components[0]);
        assert_eq!(ComponentValue{ part: 1, coins: vec![(0, 0), (1, 0), (2, 0)], value: Value::Nimber(0) }, components[1]);
        assert_eq!(ComponentValue{ part: 1, coins: vec![(0, 1), (1, 1), (2, 1)], value: Value::Nimber(0) }, components[2]);

        // Opening a chain makes just that component loony
        pos.make_move(CPosMove::new(1, 0, 1, Side::Left));
        let values: Vec<Value> = component_values(&pos).iter().map(|c| c.value).collect();
        assert_eq!(vec![Value::Nimber(1), Value::Nimber(0), Value::Loony], values);
    }

    #[test]
    fn limited_analysis() {
        let pos = CompoundPosition::from_single(p50());
        let mut calculator = NimstringCalculator::new();
        // Not enough budget: gives up, and what it cached is still correct
        assert_eq!(None, calculator.analysis_limited(&pos, 5));
        let full = calculator.analysis_limited(&pos, usize::MAX).unwrap();
        assert_eq!(calc_value_with_moves(&pos), (full.value, full.per_move.clone()));
        assert_eq!(component_values(&pos), full.components);
        // Once calculated, no new positions are needed
        assert_eq!(Some(full), calculator.analysis_limited(&pos, 0));
    }

    #[test]
    fn limited_analysis_after_aborts_matches_fresh() {
        // Giving up partway through many positions never leaves wrong values cached
        let mut calculator = NimstringCalculator::new();
        let positions = vec!(p50(), p50_top(), ex3p1(), ex3p12(), ex6p2(), ex7p2(), icelandic_game(2, 2), double_chain(4));
        for (i, pos) in positions.iter().enumerate() {
            let pos = CompoundPosition::from_single(pos.clone());
            let _ = calculator.analysis_limited(&pos, i * 3);
            let full = calculator.analysis_limited(&pos, usize::MAX).unwrap();
            assert_eq!(calc_value_with_moves(&pos), (full.value, full.per_move), "{}", pos);
            assert_eq!(component_values(&pos), full.components);
        }
    }

    #[test]
    fn compound_values() {
        let mut pos = CompoundPosition::new(vec!(make_chain(5), make_chain(5)));
        assert_eq!(false, pos.is_loony());
        let (val, _per_move) = calc_value_with_moves(&pos);
        assert_eq!(Value::Nimber(0), val);
        pos.make_move(CPosMove::new(1, 0, 0, Side::Left));
        assert_eq!(true, pos.is_loony());
        let (val, _per_move) = calc_value_with_moves(&pos);
        assert_eq!(Value::Loony, val);
    }
}
