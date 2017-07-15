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
use nimstring;
use std::cmp;
use std::collections::HashMap;

fn eval_cache(pos: &mut Position, cache: &mut HashMap<usize, isize>) -> isize {
    if let Some(&cached) = cache.get(&pos.zhash()) {
        return cached;
    }

    let moves = pos.legal_moves();
    if moves.is_empty() {
        return 0;
    }

    // If there are any captures which don't affect looniness, just go ahead and make those
    let is_loony = nimstring::is_loony(pos);
    for m in &moves {
        let captures = pos.would_capture(m.x, m.y, m.side);
        if captures == 0 {
            continue;
        }
        if !is_loony || nimstring::would_be_loony(pos, m.x, m.y, m.side) {
            pos.make_move(m.x, m.y, m.side);
            let result = captures + eval_cache(pos, cache);
            pos.undo_move(m.x, m.y, m.side);
            cache.insert(pos.zhash(), result);
            return result;
        }
        //TODO: Consider only capturing all and double-dealing in the loony case
    }

    let mut sub_vals = Vec::with_capacity(moves.len());
    for m in &moves {
        let captures = pos.would_capture(m.x, m.y, m.side);
        let sign = if captures > 0 { 1 } else { -1 };
        pos.make_move(m.x, m.y, m.side);
        sub_vals.push(captures + sign * eval_cache(pos, cache));
        pos.undo_move(m.x, m.y, m.side);
    }
    let result = sub_vals.iter().fold(sub_vals[0], |a, &v| cmp::max(a, v));
    cache.insert(pos.zhash(), result);
    result
}

// Calculate the value function of a given position
pub fn eval(pos: &Position) -> isize {
    let mut cache = HashMap::new();
    let mut pos = pos.clone();
    eval_cache(&mut pos, &mut cache)
}

#[cfg(test)]
mod test {
    use eval::*;
    use examples::*;

    #[test]
    fn eval_chain() {
        for i in 1..10 {
            let mut chain = make_chain(i);
            assert_eq!(-(i as isize), eval(&mut chain));
        }
    }

    #[test]
    fn eval_double_chain() {
        assert_eq!(0, eval(&mut double_chain(1)));
        for i in 2..10 {
            let mut pos = double_chain(i);
            assert_eq!(4 - 2*(i as isize), eval(&mut pos), "Evaluation of double chain length {}", i);
        }
    }

    #[test]
    fn eval_double_loop() {
        for i in 2..8 {
            let mut pos = double_loop(i);
            assert_eq!(8 - 4*(i as isize), eval(&mut pos), "Evaluation of double loop width {}", i);
        }
    }
}
