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
use game::Position;
use std::cmp;

// Calculate the value function of a given position
pub fn eval(pos: &mut Position) -> isize {
    let moves = pos.legal_moves();
    if moves.is_empty() {
        return 0;
    }
    let mut sub_vals = Vec::with_capacity(moves.len());
    for m in moves {
        let (c, i) = if pos.would_capture(m.x, m.y, m.side) {
            (1, 1)
        } else {
            (0, -1)
        };
        pos.make_move(m.x, m.y, m.side);
        sub_vals.push(c + i * eval(pos));
        pos.undo_move(m.x, m.y, m.side);
    }
    sub_vals.iter().fold(sub_vals[0], |a, &v| cmp::max(a, v))
}

#[cfg(test)]
mod test {
    use eval::*;
    use examples::*;

    #[test]
    fn eval_chain() {
        for i in 1..5 {
            let mut chain = make_chain(i);
            assert_eq!(-(i as isize), eval(&mut chain));
        }
    }
}
