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
    along with Foobar.  If not, see <http://www.gnu.org/licenses/>.
*/
use game::{Position, Side};
use std::collections::HashSet;
use std::fmt;

#[derive(Clone)]
#[derive(Copy)]
#[derive(PartialEq)]
#[derive(Debug)]
pub enum Value {
    Nimber(usize),
    Loony
}

impl fmt::Display for Value {
    fn fmt(self: &Value, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Value::Nimber(v) => write!(f, "*{}", v),
            &Value::Loony => write!(f, "L"),
        }
    }
}

// Indicate whether a given position is loony
pub fn is_loony(pos: &Position) -> bool {
    for i in 0..pos.width() {
        for j in 0..pos.height() {
            if pos.valency(i, j) != 1 {
                continue
            }
            for &s in [Side::Top, Side::Bottom, Side::Left, Side::Right].iter() {
                if pos.is_legal_move(i, j, s) {
                    if let Some((nx, ny)) = pos.offset(i, j, s) {
                        if pos.valency(nx, ny) == 2 {
                            // We have found a capturable coin attached to a coin of valency 2 => loony
                            return true;
                        }
                    }
                    break;
                }
            }
        }
    }
    false
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

// Calculate the Nimstring value of a given position.
// Modifies the position as it goes, but undoes its modifications before returning.
pub fn calc_value(pos: &mut Position) -> Value {
    // TODO: Optimise by caching repeated sub-positions
    // TODO: Optimise by iterating over a tighter set of moves than all legal moves
    if is_loony(pos) {
        return Value::Loony;
    }
    let legal_moves = pos.legal_moves();
    for m in &legal_moves {
        if pos.valency(m.x, m.y) == 1 {
            pos.make_move(m.x, m.y, m.side);
            let result = calc_value(pos);
            pos.undo_move(m.x, m.y, m.side);
            return result
        }
    }
    let mut options = HashSet::new();
    for m in &legal_moves {
        pos.make_move(m.x, m.y, m.side);
        if let Value::Nimber(n) = calc_value(pos) {
            options.insert(n);
        }
        pos.undo_move(m.x, m.y, m.side);
    }
    return Value::Nimber(mex(options));
}

#[cfg(test)]
mod tests {
    use nimstring::*;
    use game::*;

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

    // Create a position consisting of a single horizontal chain of a given length.
    fn make_chain(length: usize) -> Position {
        let mut pos = Position::new_game(length, 1);
        for i in 0..length {
            pos.make_move(i, 0, Side::Top);
            pos.make_move(i, 0, Side::Bottom);
        }
        pos
    }

    #[test]
    fn basic_values() {
        let mut pos = make_chain(3);
        assert!(!is_loony(&pos));
        assert_eq!(Value::Nimber(0), calc_value(&mut pos));
        pos.make_move(0, 0, Side::Left);
        assert!(is_loony(&pos));
        assert_eq!(Value::Loony, calc_value(&mut pos));
        pos.make_move(1, 0, Side::Left);
        assert!(is_loony(&pos));
        assert_eq!(Value::Loony, calc_value(&mut pos));
        pos.make_move(2, 0, Side::Left);
        assert!(!is_loony(&pos));
        assert_eq!(Value::Nimber(0), calc_value(&mut pos));
        pos.make_move(2, 0, Side::Right);
        assert!(!is_loony(&pos));
        assert_eq!(Value::Nimber(0), calc_value(&mut pos));
    }

    #[test]
    fn nonzero_value() {
        let mut pos = make_chain(7);
        pos.undo_move(3, 0, Side::Top);
        assert_eq!(Value::Nimber(1), calc_value(&mut pos));
    }
}
