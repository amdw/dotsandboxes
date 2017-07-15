/*
    Copyright 2017 Andrew Medworth <github@medworth.org.uk>

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
use game::{Position, Side, Move};
use splitter;
use std::collections::{HashMap, HashSet};
use std::fmt;
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

// If there is a coin connected to (x,y) on one of the given sides, return one such, else None.
fn connected_coin(pos: &Position, x: usize, y: usize, sides: Vec<Side>) -> Option<(usize, usize, Side)> {
    for &s in &sides {
        if pos.is_legal_move(x, y, s) {
            if let Some((nx, ny)) = pos.offset(x, y, s) {
                return Some((nx, ny, s));
            }
        }
    }
    None
}

// Indicate whether a given position is loony
pub fn is_loony(pos: &Position) -> bool {
    for x in 0..pos.width() {
        for y in 0..pos.height() {
            if pos.valency(x, y) != 1 {
                continue
            }
            if let Some((neighbour_x, neighbour_y, side)) = connected_coin(pos, x, y, Side::all()) {
                if pos.valency(neighbour_x, neighbour_y) != 2 {
                    continue
                }
                // We have found a capturable coin attached to a coin of valency 2 (o-o-?).
                // This means the position is loony unless there is a valency-1 coin
                // on the other side (o-o-o).
                let far_sides = Side::all_except(side.opposite());
                if let Some((far_x, far_y, _)) = connected_coin(pos, neighbour_x, neighbour_y, far_sides) {
                    if pos.valency(far_x, far_y) == 1 {
                        continue;
                    }
                }
                return true;
            }
        }
    }
    false
}

// Indicate whether a given move would result in a loony position
// (regardless of whether the current position is loony).
// Note that a move is loony iff it returns true here *and* it is not a capture.
// A capture is never a loony move but can return true here (e.g. capturing
// the first coin of an open 3-chain).
pub fn would_be_loony(pos: &mut Position, x: usize, y: usize, s: Side) -> bool {
    pos.make_move(x, y, s);
    let result = is_loony(pos);
    pos.undo_move(x, y, s);
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

fn calc_value(pos: &mut Position, cache: &mut HashMap<usize, Value>) -> Value {
    // TODO: Optimise by iterating over a tighter set of moves than all legal moves
    if let Some(&v) = cache.get(&pos.zhash()) {
        return v;
    }
    if is_loony(pos) {
        cache.insert(pos.zhash(), Value::Loony);
        return Value::Loony;
    }

    let legal_moves = pos.legal_moves();
    for m in &legal_moves {
        if pos.would_capture(m.x, m.y, m.side) > 0 {
            pos.make_move(m.x, m.y, m.side);
            let result = calc_value(pos, cache);
            pos.undo_move(m.x, m.y, m.side);
            cache.insert(pos.zhash(), result);
            return result
        }
    }

    // Try to split the position into independent parts which can be evaluated separately
    let parts = splitter::split(pos);
    if parts.len() > 1 {
        let mut result = Value::Nimber(0);
        for mut part in parts {
            let frag_value = calc_value(&mut part.pos, cache);
            result = result + frag_value;
        }
        cache.insert(pos.zhash(), result);
        return result;
    }

    let mut options = HashSet::new();
    for m in &legal_moves {
        pos.make_move(m.x, m.y, m.side);
        if let Value::Nimber(n) = calc_value(pos, cache) {
            options.insert(n);
        }
        pos.undo_move(m.x, m.y, m.side);
    }
    let result = Value::Nimber(mex(options));
    cache.insert(pos.zhash(), result);
    result
}

// Calculate the Nimstring value of a position, along with the values attained
// by each of the legal moves.
pub fn calc_value_with_moves(pos: &Position) -> (Value, HashMap<Move, Value>) {
    let mut cache = HashMap::new();
    let mut pos = pos.clone();
    let val = calc_value(&mut pos, &mut cache);
    let mut per_move = HashMap::new();
    for m in pos.legal_moves() {
        pos.make_move(m.x, m.y, m.side);
        per_move.insert(m, calc_value(&mut pos, &mut cache));
        pos.undo_move(m.x, m.y, m.side);
    }
    (val, per_move)
}

#[cfg(test)]
mod tests {
    use nimstring::*;
    use game::*;
    use examples::*;

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
        assert!(!is_loony(&pos));
        let mut cache = HashMap::new();
        assert_eq!(Value::Nimber(0), calc_value(&mut pos, &mut cache));
        pos.make_move(0, 0, Side::Left);
        assert!(is_loony(&pos));
        assert_eq!(Value::Loony, calc_value(&mut pos, &mut cache));
        pos.make_move(1, 0, Side::Left);
        assert!(is_loony(&pos));
        assert_eq!(Value::Loony, calc_value(&mut pos, &mut cache));
        pos.make_move(2, 0, Side::Left);
        assert!(!is_loony(&pos));
        assert_eq!(Value::Nimber(0), calc_value(&mut pos, &mut cache));
        pos.make_move(2, 0, Side::Right);
        assert!(!is_loony(&pos));
        assert_eq!(Value::Nimber(0), calc_value(&mut pos, &mut cache));
    }

    #[test]
    fn open_3loop_not_loony() {
        let mut pos = make_chain(3);
        pos.make_move(0, 0, Side::Left);
        pos.make_move(2, 0, Side::Right);
        assert_eq!(false, is_loony(&pos));
        let (val, _) = calc_value_with_moves(&pos);
        assert_eq!(Value::Nimber(0), val);
    }

    #[test]
    fn nonzero_value() {
        let mut pos = make_chain(7);
        pos.undo_move(3, 0, Side::Top);
        let (val, per_move) = calc_value_with_moves(&pos);
        assert_eq!(Value::Nimber(1), val);
        assert_eq!(&Value::Nimber(0), per_move.get(&Move{x: 3, y: 0, side: Side::Top}).unwrap());
    }

    #[test]
    fn right_capture_detection() {
        let mut pos = make_chain(5);
        pos.undo_move(3, 0, Side::Top);
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
    fn conditional_looniness() {
        let mut pos = make_chain(5);
        assert_eq!(true, would_be_loony(&mut pos, 0, 0, Side::Left));
        pos.make_move(0, 0, Side::Left);
        assert_eq!(true, is_loony(&pos));
        assert_eq!(true, would_be_loony(&mut pos, 1, 0, Side::Left));
        pos.make_move(1, 0, Side::Left);
        assert_eq!(true, is_loony(&pos));
        assert_eq!(true, would_be_loony(&mut pos, 2, 0, Side::Left));
        pos.make_move(2, 0, Side::Left);
        assert_eq!(true, is_loony(&pos));
        assert_eq!(true, would_be_loony(&mut pos, 3, 0, Side::Left));
        pos.make_move(3, 0, Side::Left);
        assert_eq!(true, is_loony(&pos));
        assert_eq!(false, would_be_loony(&mut pos, 4, 0, Side::Left));
        assert_eq!(false, would_be_loony(&mut pos, 4, 0, Side::Right));
        pos.make_move(4, 0, Side::Left);
        assert_eq!(false, is_loony(&pos));
        assert_eq!(false, would_be_loony(&mut pos, 4, 0, Side::Right));
        pos.make_move(4, 0, Side::Right);
        assert_eq!(true, pos.is_end_of_game());
        assert_eq!(false, is_loony(&mut pos));
    }
}
