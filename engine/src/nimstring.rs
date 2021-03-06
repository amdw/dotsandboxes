/*
    Copyright 2017-2020 Andrew Medworth <github@medworth.org.uk>

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
use crate::splitter::SplittablePosition;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::Hash;
use std::ops;

#[derive(Clone)]
#[derive(Copy)]
#[derive(PartialEq)]
#[derive(Debug)]
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
                if let Some((neighbour_x, neighbour_y, side)) = connected_coin(self, x, y, Side::all()) {
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
fn connected_coin(pos: &SimplePosition, x: usize, y: usize, sides: Vec<Side>) -> Option<(usize, usize, Side)> {
    for &s in &sides {
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

fn calc_value<M, P>(pos: &mut P, cache: &mut HashMap<usize, Value>) -> Value
where M: Copy, P: NimstringPosition<M> {
    // TODO: Optimise by iterating over a tighter set of moves than all legal moves
    if let Some(&v) = cache.get(&pos.zhash()) {
        return v;
    }
    if pos.is_loony() {
        cache.insert(pos.zhash(), Value::Loony);
        return Value::Loony;
    }

    let legal_moves = pos.legal_moves();
    for &m in &legal_moves {
        if pos.would_capture(m) > 0 {
            pos.make_move(m);
            let result = calc_value(pos, cache);
            pos.undo_move(m);
            cache.insert(pos.zhash(), result);
            return result
        }
    }

    // Try to split the position into independent parts which can be evaluated separately
    let parts = pos.split();
    if parts.len() > 1 {
        let mut result = Value::Nimber(0);
        for mut part in parts {
            let part_value = calc_value(&mut part, cache);
            result = result + part_value;
        }
        cache.insert(pos.zhash(), result);
        return result;
    }

    let mut options = HashSet::new();
    for &m in &legal_moves {
        pos.make_move(m);
        if let Value::Nimber(n) = calc_value(pos, cache) {
            options.insert(n);
        }
        pos.undo_move(m);
    }
    let result = Value::Nimber(mex(options));
    cache.insert(pos.zhash(), result);
    result
}

// Calculate the Nimstring value of a position, along with the values attained
// by each of the legal moves.
pub fn calc_value_with_moves<M, P>(pos: &P) -> (Value, HashMap<M, Value>)
where M: Hash + Eq + Copy, P: NimstringPosition<M> + Clone {
    let mut cache = HashMap::new();
    let mut pos = pos.clone();
    let val = calc_value(&mut pos, &mut cache);
    let mut per_move = HashMap::new();
    for m in pos.legal_moves() {
        pos.make_move(m);
        per_move.insert(m, calc_value(&mut pos, &mut cache));
        pos.undo_move(m);
    }
    (val, per_move)
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
        assert_eq!(Value::Nimber(0), calc_value(&mut pos, &mut cache));
        pos.make_move(Move{x: 0, y: 0, side: Side::Left});
        assert!(pos.is_loony());
        assert_eq!(Value::Loony, calc_value(&mut pos, &mut cache));
        pos.make_move(Move{x: 1, y: 0, side: Side::Left});
        assert!(pos.is_loony());
        assert_eq!(Value::Loony, calc_value(&mut pos, &mut cache));
        pos.make_move(Move{x: 2, y: 0, side: Side::Left});
        assert!(!pos.is_loony());
        assert_eq!(Value::Nimber(0), calc_value(&mut pos, &mut cache));
        pos.make_move(Move{x: 2, y: 0, side: Side::Right});
        assert!(!pos.is_loony());
        assert_eq!(Value::Nimber(0), calc_value(&mut pos, &mut cache));
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
